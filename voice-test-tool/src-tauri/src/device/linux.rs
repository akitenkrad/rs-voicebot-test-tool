//! Linux virtual device manager implementation.
//!
//! On Linux with PulseAudio, virtual audio devices (null-sink modules) can be
//! created and destroyed programmatically using the `pactl` command-line tool.
//! This module uses `std::process::Command` to interact with PulseAudio.

use std::process::Command;

use cpal::traits::{DeviceTrait, HostTrait};
use tracing;

use crate::error::DeviceError;
use crate::types::{DeviceInfo, DeviceStatus};

use super::VirtualDeviceManager;

/// Linux device manager that creates/destroys PulseAudio null-sink modules.
pub struct LinuxDeviceManager {
    /// Name of the currently active virtual device, if any.
    active_device_name: Option<String>,
    /// PulseAudio module ID returned by `pactl load-module`, used for cleanup.
    module_id: Option<u32>,
}

impl LinuxDeviceManager {
    pub fn new() -> Self {
        tracing::info!("Initializing Linux device manager");
        Self {
            active_device_name: None,
            module_id: None,
        }
    }

    /// Check if PulseAudio is available by running `pactl info`.
    fn is_pulseaudio_available() -> bool {
        Command::new("pactl")
            .arg("info")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Parse the list of PulseAudio sinks from `pactl list sinks short`.
    fn list_pactl_sinks() -> Result<Vec<(String, String)>, DeviceError> {
        let output = Command::new("pactl")
            .args(["list", "sinks", "short"])
            .output()
            .map_err(|e| {
                DeviceError::NotFound(format!("Failed to run pactl list sinks: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(DeviceError::NotFound(format!(
                "pactl list sinks failed: {}",
                stderr.trim()
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut sinks = Vec::new();

        // Format: <index>\t<name>\t<module>\t<sample_spec>\t<state>
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 2 {
                let id = parts[0].trim().to_string();
                let name = parts[1].trim().to_string();
                sinks.push((id, name));
            }
        }

        Ok(sinks)
    }
}

impl VirtualDeviceManager for LinuxDeviceManager {
    fn create_device(&mut self, name: &str) -> Result<DeviceInfo, DeviceError> {
        tracing::info!(name = %name, "Creating PulseAudio null-sink virtual device");

        if !Self::is_pulseaudio_available() {
            return Err(DeviceError::CreationFailed(
                "PulseAudio is not available. Please ensure PulseAudio is installed and running."
                    .to_string(),
            ));
        }

        // If we already have a module loaded, destroy it first.
        if self.module_id.is_some() {
            tracing::info!("Destroying existing virtual device before creating a new one");
            self.destroy_device()?;
        }

        // Create the null-sink module via pactl.
        // The sink_name is sanitized to be a valid PulseAudio identifier (no spaces).
        let sink_name = name.replace(' ', "_");
        let output = Command::new("pactl")
            .args([
                "load-module",
                "module-null-sink",
                &format!("sink_name={}", sink_name),
                &format!("sink_properties=device.description={}", name),
            ])
            .output()
            .map_err(|e| {
                DeviceError::CreationFailed(format!("Failed to run pactl load-module: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(DeviceError::CreationFailed(format!(
                "pactl load-module failed: {}",
                stderr.trim()
            )));
        }

        // Parse the module ID from stdout (pactl prints the numeric module ID).
        let stdout = String::from_utf8_lossy(&output.stdout);
        let module_id: u32 = stdout.trim().parse().map_err(|e| {
            DeviceError::CreationFailed(format!(
                "Failed to parse module ID from pactl output '{}': {}",
                stdout.trim(),
                e
            ))
        })?;

        tracing::info!(module_id = module_id, sink_name = %sink_name, "Created PulseAudio null-sink");

        self.active_device_name = Some(name.to_string());
        self.module_id = Some(module_id);

        Ok(DeviceInfo {
            id: sink_name,
            name: name.to_string(),
            is_virtual: true,
            is_default: false,
            is_active: true,
        })
    }

    fn destroy_device(&mut self) -> Result<(), DeviceError> {
        if let Some(module_id) = self.module_id.take() {
            tracing::info!(module_id = module_id, "Unloading PulseAudio module");

            let output = Command::new("pactl")
                .args(["unload-module", &module_id.to_string()])
                .output()
                .map_err(|e| {
                    DeviceError::NotFound(format!("Failed to run pactl unload-module: {}", e))
                })?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                tracing::warn!(
                    module_id = module_id,
                    stderr = %stderr.trim(),
                    "pactl unload-module returned non-zero (module may already be gone)"
                );
                // Not returning an error here -- the module may already have been unloaded.
            }
        }

        self.active_device_name = None;
        Ok(())
    }

    fn list_devices(&self) -> Result<Vec<DeviceInfo>, DeviceError> {
        let mut result = Vec::new();

        // First, list PulseAudio sinks via pactl for richer info.
        let pactl_sinks = Self::list_pactl_sinks().unwrap_or_default();
        let pactl_names: std::collections::HashSet<String> =
            pactl_sinks.iter().map(|(_, name)| name.clone()).collect();

        // Also enumerate via cpal for completeness.
        let host = cpal::default_host();
        let default_output_name = host
            .default_output_device()
            .and_then(|d| d.name().ok());

        let devices = host.output_devices().map_err(|e| {
            DeviceError::NotFound(format!("Failed to enumerate output devices: {}", e))
        })?;

        let mut seen_names = std::collections::HashSet::new();

        for device in devices {
            let name = match device.name() {
                Ok(n) => n,
                Err(e) => {
                    tracing::warn!("Skipping device with unreadable name: {}", e);
                    continue;
                }
            };

            if seen_names.contains(&name) {
                continue;
            }
            seen_names.insert(name.clone());

            // A device is considered virtual if it was created by our null-sink
            // or if it shows up only in pactl as a null-sink.
            let is_virtual = self
                .active_device_name
                .as_ref()
                .map(|a| name.contains(&a.replace(' ', "_")))
                .unwrap_or(false);

            let is_default = default_output_name
                .as_ref()
                .map(|d| d == &name)
                .unwrap_or(false);

            let is_active = self
                .active_device_name
                .as_ref()
                .map(|a| name.contains(&a.replace(' ', "_")))
                .unwrap_or(false);

            result.push(DeviceInfo {
                id: name.clone(),
                name,
                is_virtual,
                is_default,
                is_active,
            });
        }

        // Add pactl sinks that cpal might not have picked up.
        for (id, sink_name) in &pactl_sinks {
            if !seen_names.contains(sink_name) {
                let is_active = self
                    .active_device_name
                    .as_ref()
                    .map(|a| sink_name.contains(&a.replace(' ', "_")))
                    .unwrap_or(false);

                result.push(DeviceInfo {
                    id: id.clone(),
                    name: sink_name.clone(),
                    is_virtual: is_active,
                    is_default: false,
                    is_active,
                });
                seen_names.insert(sink_name.clone());
            }
        }

        tracing::debug!(count = result.len(), "Listed audio output devices");
        Ok(result)
    }

    fn detect_virtual_device(&self) -> Result<Option<DeviceInfo>, DeviceError> {
        // Check if our named sink still exists in the pactl sink list.
        let active_name = match &self.active_device_name {
            Some(name) => name.clone(),
            None => return Ok(None),
        };

        let sink_name = active_name.replace(' ', "_");
        let sinks = Self::list_pactl_sinks()?;

        for (id, name) in sinks {
            if name.contains(&sink_name) {
                return Ok(Some(DeviceInfo {
                    id,
                    name: active_name,
                    is_virtual: true,
                    is_default: false,
                    is_active: true,
                }));
            }
        }

        Ok(None)
    }

    fn get_status(&self) -> Result<DeviceStatus, DeviceError> {
        match &self.active_device_name {
            Some(name) => {
                // Verify the sink still exists
                let still_available = self.detect_virtual_device()?.is_some();

                if still_available {
                    Ok(DeviceStatus {
                        device_name: name.clone(),
                        is_active: true,
                        is_default: false,
                        error: None,
                    })
                } else {
                    Ok(DeviceStatus {
                        device_name: name.clone(),
                        is_active: false,
                        is_default: false,
                        error: Some(format!(
                            "PulseAudio null-sink '{}' is no longer available (module may have been unloaded)",
                            name
                        )),
                    })
                }
            }
            None => Ok(DeviceStatus {
                device_name: String::new(),
                is_active: false,
                is_default: false,
                error: Some("No virtual device active".to_string()),
            }),
        }
    }

    fn active_device_name(&self) -> Option<String> {
        self.active_device_name.clone()
    }
}
