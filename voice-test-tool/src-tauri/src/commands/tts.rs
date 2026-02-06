use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::sync::Arc;

use chrono::Utc;
use tauri::{AppHandle, State};

use crate::state::AppState;
use crate::tts::{
    csv_parser, generator, TtsTestCase, TtsTestRunner, TtsTestSession, TtsTestStatus,
};

/// Load TTS test cases from a CSV file.
///
/// Parses the CSV and returns a list of test cases with IDs and text.
#[tauri::command]
pub async fn load_tts_csv(path: String) -> Result<Vec<TtsTestCase>, String> {
    let file_path = Path::new(&path);

    let test_cases = csv_parser::parse_csv(file_path).map_err(|e| e.to_string())?;

    tracing::info!(
        "Loaded {} TTS test cases from: {}",
        test_cases.len(),
        path
    );

    Ok(test_cases)
}

/// Generate TTS audio for a single text preview.
///
/// Uses the stored TTS configuration from AppState.
/// Returns the audio bytes as a Vec<u8>.
#[tauri::command]
pub async fn preview_tts(text: String, state: State<'_, AppState>) -> Result<Vec<u8>, String> {
    let tts_config = {
        let config = state.config.lock();
        config.tts.clone()
    };

    tracing::info!(
        "Generating TTS preview for text: {}",
        &text[..text.len().min(50)]
    );

    let bytes = generator::generate_tts(&text, &tts_config)
        .await
        .map_err(|e| e.to_string())?;

    tracing::info!("Generated {} bytes of TTS audio", bytes.len());

    Ok(bytes)
}

/// Start a TTS test execution.
///
/// Loads test cases from the CSV file and begins sequential execution.
/// Returns the session ID for tracking progress.
#[tauri::command]
pub async fn start_tts_test(
    csv_path: String,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<String, String> {
    let file_path = Path::new(&csv_path);

    // Parse the CSV file
    let test_cases = csv_parser::parse_csv(file_path).map_err(|e| e.to_string())?;

    if test_cases.is_empty() {
        return Err("No test cases found in CSV file".to_string());
    }

    // Get configurations
    let (tts_config, recording_config) = {
        let config = state.config.lock();
        let rec_config = state.recording_config.lock();
        (config.tts.clone(), rec_config.clone())
    };

    // Generate session ID
    let session_id = uuid::Uuid::new_v4().to_string();

    // Create the test session
    let session = TtsTestSession {
        id: session_id.clone(),
        csv_path: PathBuf::from(&csv_path),
        test_cases,
        results: Vec::new(),
        config: tts_config,
        recording_config,
        status: TtsTestStatus::Pending,
        started_at: Utc::now(),
        ended_at: None,
    };

    // Create the runner
    let runner = TtsTestRunner::new(session);

    // Store the runner in state
    {
        let mut sessions = state.tts_sessions.lock();
        sessions.insert(session_id.clone(), runner);
    }

    // Clone Arc references for the spawned task
    let player_arc = Arc::clone(&state.player);
    let tts_sessions_arc = Arc::clone(&state.tts_sessions);
    let session_id_clone = session_id.clone();

    tracing::info!(
        "Starting TTS test session: {} from {}",
        session_id,
        csv_path
    );

    // Get the test data we need before spawning
    let (test_cases_clone, tts_config_clone, recording_config_clone, cancel_flag, pause_flag) = {
        let sessions = state.tts_sessions.lock();
        let runner = sessions
            .get(&session_id)
            .ok_or_else(|| format!("Runner not found: {}", session_id))?;
        let session = runner.get_session();
        (
            session.test_cases.clone(),
            session.config.clone(),
            session.recording_config.clone(),
            runner.cancel_flag(),
            runner.pause_flag(),
        )
    };

    // Spawn the execution task
    tokio::spawn(async move {
        let result = crate::tts::runner::execute_tts_test(
            session_id_clone.clone(),
            test_cases_clone,
            tts_config_clone,
            recording_config_clone,
            cancel_flag,
            pause_flag,
            player_arc,
            tts_sessions_arc.clone(),
            Some(app_handle),
        )
        .await;

        match result {
            Ok(()) => {
                tracing::info!("TTS test session completed: {}", session_id_clone);
            }
            Err(e) => {
                tracing::error!("TTS test session failed: {}: {}", session_id_clone, e);
            }
        }
    });

    Ok(session_id)
}

/// Pause a running TTS test.
#[tauri::command]
pub async fn pause_tts_test(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let sessions = state.tts_sessions.lock();
    let runner = sessions
        .get(&session_id)
        .ok_or_else(|| format!("TTS test session not found: {}", session_id))?;

    runner.pause_flag().store(true, Ordering::SeqCst);
    tracing::info!("Pausing TTS test session: {}", session_id);

    Ok(())
}

/// Resume a paused TTS test.
#[tauri::command]
pub async fn resume_tts_test(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let sessions = state.tts_sessions.lock();
    let runner = sessions
        .get(&session_id)
        .ok_or_else(|| format!("TTS test session not found: {}", session_id))?;

    runner.pause_flag().store(false, Ordering::SeqCst);
    tracing::info!("Resuming TTS test session: {}", session_id);

    Ok(())
}

/// Abort a running TTS test.
#[tauri::command]
pub async fn abort_tts_test(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let sessions = state.tts_sessions.lock();
    let runner = sessions
        .get(&session_id)
        .ok_or_else(|| format!("TTS test session not found: {}", session_id))?;

    runner.cancel_flag().store(true, Ordering::SeqCst);
    tracing::info!("Aborting TTS test session: {}", session_id);

    Ok(())
}

/// Get the status of a TTS test session.
#[tauri::command]
pub async fn get_tts_test_status(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<TtsTestStatus, String> {
    let sessions = state.tts_sessions.lock();
    let runner = sessions
        .get(&session_id)
        .ok_or_else(|| format!("TTS test session not found: {}", session_id))?;

    let session = runner.get_session();
    Ok(session.status)
}

/// Get the full session data for a TTS test.
#[tauri::command]
pub async fn get_tts_test_session(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<TtsTestSession, String> {
    let sessions = state.tts_sessions.lock();
    let runner = sessions
        .get(&session_id)
        .ok_or_else(|| format!("TTS test session not found: {}", session_id))?;

    Ok(runner.get_session())
}

/// Export a TTS test session to a ZIP archive.
///
/// Creates a ZIP file containing metadata and audio files.
/// Returns the path to the created ZIP file.
#[tauri::command]
pub async fn export_tts_test(
    session_id: String,
    output_path: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let sessions = state.tts_sessions.lock();
    let runner = sessions
        .get(&session_id)
        .ok_or_else(|| format!("TTS test session not found: {}", session_id))?;

    let session = runner.get_session();
    let path = Path::new(&output_path);

    tracing::info!(
        "Exporting TTS test session {} to {}",
        session_id,
        output_path
    );

    crate::tts::export_session_to_zip(&session, path)
}
