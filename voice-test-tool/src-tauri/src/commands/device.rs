#![allow(unused)]

use tauri::State;
use tracing;

use crate::state::AppState;
use crate::types::{DeviceInfo, DeviceStatus};

/// Create a virtual audio device (or detect a pre-installed one).
///
/// On macOS: detects BlackHole / Soundflower.
/// On Windows: detects VB-Audio Virtual Cable / VoiceMeeter.
/// On Linux: creates a PulseAudio null-sink module via `pactl`.
#[tauri::command]
pub async fn create_virtual_device(
    name: String,
    state: State<'_, AppState>,
) -> Result<DeviceInfo, String> {
    tracing::info!(name = %name, "create_virtual_device command invoked");

    let device = {
        let mut manager = state.device_manager.lock();
        manager.create_device(&name).map_err(|e| e.to_string())?
    };

    // Store the active device info in shared state.
    *state.active_device.lock() = Some(device.clone());

    tracing::info!(device_name = %device.name, "Virtual device activated");
    Ok(device)
}

/// Destroy the active virtual audio device.
///
/// On macOS/Windows this just clears the active reference (system drivers
/// cannot be removed programmatically). On Linux this unloads the PulseAudio
/// null-sink module.
#[tauri::command]
pub async fn destroy_virtual_device(
    state: State<'_, AppState>,
) -> Result<(), String> {
    tracing::info!("destroy_virtual_device command invoked");

    {
        let mut manager = state.device_manager.lock();
        manager.destroy_device().map_err(|e| e.to_string())?;
    }

    *state.active_device.lock() = None;

    tracing::info!("Virtual device destroyed / reference cleared");
    Ok(())
}

/// List all available audio output devices.
#[tauri::command]
pub async fn list_audio_devices(
    state: State<'_, AppState>,
) -> Result<Vec<DeviceInfo>, String> {
    tracing::debug!("list_audio_devices command invoked");

    let manager = state.device_manager.lock();
    let devices = manager.list_devices().map_err(|e| e.to_string())?;

    tracing::debug!(count = devices.len(), "Returning audio device list");
    Ok(devices)
}

/// Store a device ID as the preferred default input device.
///
/// Note: Actually changing the system-level default input device requires
/// OS-specific APIs (e.g., CoreAudio on macOS, registry changes on Windows,
/// `pactl set-default-source` on Linux) which are out of scope for now.
/// This command records the preference in application state.
#[tauri::command]
pub async fn set_default_input_device(
    device_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    tracing::info!(device_id = %device_id, "set_default_input_device command invoked");

    // Update the active device's is_default flag if it matches.
    let mut active_device = state.active_device.lock();
    if let Some(ref mut device) = *active_device {
        if device.id == device_id {
            device.is_default = true;
            tracing::info!(device_id = %device_id, "Marked active device as default");
        }
    }

    Ok(())
}

/// Get the current virtual device status.
#[tauri::command]
pub async fn get_device_status(
    state: State<'_, AppState>,
) -> Result<DeviceStatus, String> {
    tracing::debug!("get_device_status command invoked");

    let manager = state.device_manager.lock();
    let status = manager.get_status().map_err(|e| e.to_string())?;

    Ok(status)
}
