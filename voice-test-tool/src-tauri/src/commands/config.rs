#![allow(unused)]

use tauri::State;

use crate::config::AppConfig;
use crate::state::AppState;

/// Get the current application configuration
#[tauri::command]
pub async fn get_config(
    state: State<'_, AppState>,
) -> Result<AppConfig, String> {
    let config = state.config.lock().clone();
    Ok(config)
}

/// Update the application configuration
#[tauri::command]
pub async fn update_config(
    config: AppConfig,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // Save to disk
    crate::config::save_config(&config).map_err(|e| e.to_string())?;
    // Update in-memory state
    *state.config.lock() = config;
    Ok(())
}
