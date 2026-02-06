use std::path::Path;

use tauri::State;

use crate::logging::reporter::TestSession;
use crate::state::AppState;
use crate::types::LogEntry;

/// Get logs for a specific test session
#[tauri::command]
pub async fn get_session_logs(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<LogEntry>, String> {
    let logger = state.test_logger.lock();
    let logs = logger.get_session_logs(&session_id);
    Ok(logs)
}

/// Export a test report in the specified format
#[tauri::command]
pub async fn export_report(
    session_id: String,
    format: String,
    output_path: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let path = Path::new(&output_path);
    let logger = state.test_logger.lock();

    match format.as_str() {
        "json" => logger
            .export_json(&session_id, path)
            .map_err(|e| e.to_string())?,
        "csv" => logger
            .export_csv(&session_id, path)
            .map_err(|e| e.to_string())?,
        "html" => logger
            .export_html(&session_id, path)
            .map_err(|e| e.to_string())?,
        other => {
            return Err(format!(
                "Unsupported export format: {}. Use 'json', 'csv', or 'html'.",
                other
            ))
        }
    }

    tracing::info!("Exported {} report to: {}", format, output_path);
    Ok(output_path)
}

/// List all test sessions
#[tauri::command]
pub async fn list_test_sessions(
    state: State<'_, AppState>,
) -> Result<Vec<TestSession>, String> {
    let logger = state.test_logger.lock();
    Ok(logger.list_sessions())
}

/// Get a specific test session by ID
#[tauri::command]
pub async fn get_test_session(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<Option<TestSession>, String> {
    let logger = state.test_logger.lock();
    Ok(logger.get_session(&session_id))
}
