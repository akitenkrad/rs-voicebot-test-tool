//! macOS virtual device manager implementation.
//!
//! On macOS, virtual audio devices cannot be created programmatically.
//! Instead, this module detects pre-installed virtual audio drivers
//! such as BlackHole or Soundflower via `cpal` device enumeration.

use cpal::traits::{DeviceTrait, HostTrait};
use tracing;

use crate::error::DeviceError;
use crate::types::{DeviceInfo, DeviceStatus};

use super::VirtualDeviceManager;

/// Known virtual audio device name patterns on macOS.
///
/// Checked with `contains()` against `cpal` device names.
/// Add new virtual audio driver names here as needed.
const VIRTUAL_DEVICE_PATTERNS: &[&str] = &[
    "BlackHole",
    "Soundflower",
    "Background Music",
    "Loopback",
    "Virtual",
];

/// macOS device manager that detects BlackHole / Soundflower.
pub struct MacOSDeviceManager {
    /// Name of the currently active virtual device, if any.
    active_device_name: Option<String>,
}

impl MacOSDeviceManager {
    pub fn new() -> Self {
        tracing::info!("Initializing macOS device manager");
        Self {
            active_device_name: None,
        }
    }

    /// Check whether a device name matches any known virtual audio device pattern.
    fn is_virtual_device(name: &str) -> bool {
        VIRTUAL_DEVICE_PATTERNS
            .iter()
            .any(|pattern| name.contains(pattern))
    }
}

impl VirtualDeviceManager for MacOSDeviceManager {
    fn create_device(&mut self, name: &str) -> Result<DeviceInfo, DeviceError> {
        tracing::info!(name = %name, "Attempting to find virtual audio device on macOS");

        // On macOS we cannot create virtual devices programmatically.
        // We search for a pre-installed one (BlackHole / Soundflower).
        if let Some(device) = self.detect_virtual_device()? {
            tracing::info!(device_name = %device.name, "Found virtual audio device");
            self.active_device_name = Some(device.name.clone());
            return Ok(DeviceInfo {
                is_active: true,
                ..device
            });
        }

        Err(DeviceError::CreationFailed(
            "No virtual audio device found on macOS. \
             Please install BlackHole (https://existential.audio/blackhole/) \
             or Soundflower to use virtual audio routing."
                .to_string(),
        ))
    }

    fn destroy_device(&mut self) -> Result<(), DeviceError> {
        // On macOS, virtual devices are system-level drivers.
        // We only clear the active reference.
        if let Some(ref name) = self.active_device_name {
            tracing::info!(device_name = %name, "Releasing virtual device reference");
        }
        self.active_device_name = None;
        Ok(())
    }

    fn list_devices(&self) -> Result<Vec<DeviceInfo>, DeviceError> {
        let host = cpal::default_host();

        let default_output_name = host
            .default_output_device()
            .and_then(|d| d.name().ok());

        let devices = host.output_devices().map_err(|e| {
            DeviceError::NotFound(format!("Failed to enumerate output devices: {}", e))
        })?;

        let mut result = Vec::new();
        for device in devices {
            let name = match device.name() {
                Ok(n) => n,
                Err(e) => {
                    tracing::warn!("Skipping device with unreadable name: {}", e);
                    continue;
                }
            };

            let is_virtual = Self::is_virtual_device(&name);
            let is_default = default_output_name
                .as_ref()
                .map(|d| d == &name)
                .unwrap_or(false);
            let is_active = self
                .active_device_name
                .as_ref()
                .map(|a| a == &name)
                .unwrap_or(false);

            result.push(DeviceInfo {
                id: name.clone(),
                name,
                is_virtual,
                is_default,
                is_active,
            });
        }

        tracing::debug!(count = result.len(), "Listed audio output devices");
        Ok(result)
    }

    fn detect_virtual_device(&self) -> Result<Option<DeviceInfo>, DeviceError> {
        let host = cpal::default_host();

        let devices = host.output_devices().map_err(|e| {
            DeviceError::NotFound(format!("Failed to enumerate output devices: {}", e))
        })?;

        let default_output_name = host
            .default_output_device()
            .and_then(|d| d.name().ok());

        for device in devices {
            let name = match device.name() {
                Ok(n) => n,
                Err(_) => continue,
            };

            if Self::is_virtual_device(&name) {
                let is_default = default_output_name
                    .as_ref()
                    .map(|d| d == &name)
                    .unwrap_or(false);
                let is_active = self
                    .active_device_name
                    .as_ref()
                    .map(|a| a == &name)
                    .unwrap_or(false);

                return Ok(Some(DeviceInfo {
                    id: name.clone(),
                    name,
                    is_virtual: true,
                    is_default,
                    is_active,
                }));
            }
        }

        Ok(None)
    }

    fn get_status(&self) -> Result<DeviceStatus, DeviceError> {
        match &self.active_device_name {
            Some(name) => {
                // Verify the device is still present
                let still_available = self
                    .detect_virtual_device()?
                    .map(|d| d.name == *name)
                    .unwrap_or(false);

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
                            "Virtual device '{}' is no longer available",
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
