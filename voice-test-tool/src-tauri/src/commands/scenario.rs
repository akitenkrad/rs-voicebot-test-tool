use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tauri::State;

use crate::scenario::executor::{self, ScenarioExecStatus, ScenarioExecutionState};
use crate::scenario::parser;
use crate::state::AppState;
use crate::types::{ScenarioData, ScenarioInfo, ScenarioProgressInfo};

/// Load a scenario from a file
#[tauri::command]
pub async fn load_scenario(
    path: String,
    state: State<'_, AppState>,
) -> Result<ScenarioInfo, String> {
    let file_path = Path::new(&path);

    tracing::info!("Loading scenario from: {}", path);

    // Parse the scenario file
    let scenario_file = parser::parse_scenario_file(file_path)
        .map_err(|e| e.to_string())?;

    // Convert to ScenarioInfo with IDs and resolved paths
    let scenario = parser::scenario_file_to_info(&scenario_file, file_path);

    // Store in state
    state.scenarios.lock().push(scenario.clone());

    tracing::info!(
        "Loaded scenario: {} ({} turns)",
        scenario.name,
        scenario.turns.len()
    );

    Ok(scenario)
}

/// Save a scenario to a file
#[tauri::command]
pub async fn save_scenario(
    scenario: ScenarioData,
    path: String,
    _state: State<'_, AppState>,
) -> Result<(), String> {
    let file_path = Path::new(&path);

    tracing::info!("Saving scenario to: {}", path);

    // Convert ScenarioData to ScenarioFile
    let scenario_file = parser::scenario_data_to_file(&scenario);

    // Save to file
    parser::save_scenario_file(&scenario_file, file_path)
        .map_err(|e| e.to_string())?;

    tracing::info!("Scenario saved successfully: {}", path);

    Ok(())
}

/// Execute a scenario and return the session ID
#[tauri::command]
pub async fn execute_scenario(
    scenario_id: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // Find the scenario
    let scenario = {
        let scenarios = state.scenarios.lock();
        scenarios
            .iter()
            .find(|s| s.id == scenario_id)
            .cloned()
            .ok_or_else(|| format!("Scenario not found: {}", scenario_id))?
    };

    if scenario.turns.is_empty() {
        return Err("Scenario has no turns".to_string());
    }

    // Create a test session via TestLogger
    let session_id = {
        let mut logger = state.test_logger.lock();
        logger.create_session(&scenario.id, &scenario.name)
    };

    // Create execution state
    let cancel = Arc::new(AtomicBool::new(false));
    let pause = Arc::new(AtomicBool::new(false));
    let total_turns = scenario.turns.len();

    let exec_state = ScenarioExecutionState {
        session_id: session_id.clone(),
        scenario_id: scenario_id.clone(),
        current_turn: 0,
        total_turns,
        status: ScenarioExecStatus::Running,
        cancel: Arc::clone(&cancel),
        pause: Arc::clone(&pause),
        current_file_name: None,
    };

    // Store execution state
    {
        let mut execs = state.scenario_executions.lock();
        execs.insert(session_id.clone(), exec_state);
    }

    // Track active session
    state.active_sessions.lock().push(session_id.clone());

    tracing::info!(
        "Executing scenario: {} (session={})",
        scenario.name,
        session_id
    );

    // Clone Arc references for the spawned task
    let player_arc = Arc::clone(&state.player);
    let decoded_cache_arc = Arc::clone(&state.decoded_cache);
    let playback_state_arc = Arc::clone(&state.playback_state);
    let test_logger_arc = Arc::clone(&state.test_logger);
    let executions_arc = Arc::clone(&state.scenario_executions);
    let active_sessions_arc = Arc::clone(&state.active_sessions);
    let session_id_clone = session_id.clone();

    // Spawn the execution task
    tokio::spawn(async move {
        let result = executor::run_scenario(
            scenario,
            session_id_clone.clone(),
            cancel,
            pause,
            player_arc,
            decoded_cache_arc,
            playback_state_arc,
            test_logger_arc,
            executions_arc,
        )
        .await;

        // Remove from active sessions
        {
            let mut sessions = active_sessions_arc.lock();
            sessions.retain(|s| s != &session_id_clone);
        }

        match result {
            Ok(()) => tracing::info!("Scenario execution task finished: {}", session_id_clone),
            Err(e) => tracing::error!("Scenario execution task failed: {}: {}", session_id_clone, e),
        }
    });

    Ok(session_id)
}

/// Pause scenario execution
#[tauri::command]
pub async fn pause_scenario(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let execs = state.scenario_executions.lock();
    let exec = execs
        .get(&session_id)
        .ok_or_else(|| format!("Execution not found: {}", session_id))?;

    exec.pause.store(true, Ordering::SeqCst);
    tracing::info!("Pausing scenario execution: {}", session_id);

    Ok(())
}

/// Resume scenario execution
#[tauri::command]
pub async fn resume_scenario(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let execs = state.scenario_executions.lock();
    let exec = execs
        .get(&session_id)
        .ok_or_else(|| format!("Execution not found: {}", session_id))?;

    exec.pause.store(false, Ordering::SeqCst);
    tracing::info!("Resuming scenario execution: {}", session_id);

    Ok(())
}

/// Abort scenario execution
#[tauri::command]
pub async fn abort_scenario(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    {
        let execs = state.scenario_executions.lock();
        if let Some(exec) = execs.get(&session_id) {
            exec.cancel.store(true, Ordering::SeqCst);
            tracing::info!("Aborting scenario execution: {}", session_id);
        } else {
            return Err(format!("Execution not found: {}", session_id));
        }
    }

    // Remove from active sessions
    let mut sessions = state.active_sessions.lock();
    sessions.retain(|s| s != &session_id);

    Ok(())
}

/// List all loaded scenarios
#[tauri::command]
pub async fn list_scenarios(state: State<'_, AppState>) -> Result<Vec<ScenarioInfo>, String> {
    let scenarios = state.scenarios.lock();
    Ok(scenarios.clone())
}

/// Delete a scenario by ID
#[tauri::command]
pub async fn delete_scenario(
    scenario_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut scenarios = state.scenarios.lock();
    let original_len = scenarios.len();
    scenarios.retain(|s| s.id != scenario_id);

    if scenarios.len() == original_len {
        return Err(format!("Scenario not found: {}", scenario_id));
    }

    tracing::info!("Deleted scenario: {}", scenario_id);
    Ok(())
}

/// Get the execution status of a running scenario
#[tauri::command]
pub async fn get_scenario_execution_status(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<Option<ScenarioProgressInfo>, String> {
    let execs = state.scenario_executions.lock();

    match execs.get(&session_id) {
        Some(exec) => Ok(Some(ScenarioProgressInfo {
            session_id: exec.session_id.clone(),
            scenario_id: exec.scenario_id.clone(),
            current_turn: exec.current_turn,
            total_turns: exec.total_turns,
            status: exec.status.as_str().to_string(),
            current_file_name: exec.current_file_name.clone(),
        })),
        None => Ok(None),
    }
}
