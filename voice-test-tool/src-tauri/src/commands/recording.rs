//! Tauri commands for audio recording functionality.

use tauri::State;

use crate::audio::{InputDeviceInfo, RecordingConfig};
use crate::state::AppState;

/// List available input devices for recording.
#[tauri::command]
pub fn list_input_devices() -> Result<Vec<InputDeviceInfo>, String> {
    crate::audio::AudioRecorder::list_input_devices()
}

/// Get the current recording configuration.
#[tauri::command]
pub fn get_recording_config(state: State<'_, AppState>) -> Result<RecordingConfig, String> {
    let config = state.recording_config.lock();
    Ok(config.clone())
}

/// Update the recording configuration.
#[tauri::command]
pub fn update_recording_config(
    config: RecordingConfig,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut current_config = state.recording_config.lock();
    *current_config = config;
    tracing::info!("Recording configuration updated");
    Ok(())
}
