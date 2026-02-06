//! Scenario execution engine
//!
//! Handles executing test scenarios with pause/resume/cancel support.
//! Similar to playlist playback but with test-specific features such as
//! logging events and managing test sessions.

use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use parking_lot::Mutex;

use crate::audio::decoder::{self, DecodedAudio};
use crate::audio::player::{AudioPlayer, PlayerCommand};
use crate::logging::reporter::TestLogger;
use crate::types::{PlaybackState, ScenarioInfo};

/// Current status of a scenario execution
#[derive(Debug, Clone, PartialEq)]
pub enum ScenarioExecStatus {
    Running,
    Paused,
    Completed,
    Failed(String),
    Aborted,
}

impl ScenarioExecStatus {
    /// Convert to a string representation for serialization
    pub fn as_str(&self) -> &str {
        match self {
            ScenarioExecStatus::Running => "running",
            ScenarioExecStatus::Paused => "paused",
            ScenarioExecStatus::Completed => "completed",
            ScenarioExecStatus::Failed(_) => "failed",
            ScenarioExecStatus::Aborted => "aborted",
        }
    }
}

/// State tracking for a scenario execution
pub struct ScenarioExecutionState {
    pub session_id: String,
    pub scenario_id: String,
    pub current_turn: usize,
    pub total_turns: usize,
    pub status: ScenarioExecStatus,
    pub cancel: Arc<AtomicBool>,
    pub pause: Arc<AtomicBool>,
    pub current_file_name: Option<String>,
}

/// Execute a scenario asynchronously.
///
/// This function iterates through all turns in the scenario, playing each
/// audio file with the specified delays, while supporting pause/resume and
/// cancellation.
pub async fn run_scenario(
    scenario: ScenarioInfo,
    session_id: String,
    cancel: Arc<AtomicBool>,
    pause: Arc<AtomicBool>,
    player_arc: Arc<Mutex<Option<AudioPlayer>>>,
    _decoded_cache_arc: Arc<Mutex<HashMap<String, DecodedAudio>>>,
    playback_state_arc: Arc<Mutex<PlaybackState>>,
    test_logger_arc: Arc<Mutex<TestLogger>>,
    executions_arc: Arc<Mutex<HashMap<String, ScenarioExecutionState>>>,
) -> Result<(), String> {
    let total_turns = scenario.turns.len();

    tracing::info!(
        "Starting scenario execution: {} ({} turns), session={}",
        scenario.name,
        total_turns,
        session_id
    );

    {
        let mut logger = test_logger_arc.lock();
        logger.log_event(
            &session_id,
            "info",
            &format!("Starting scenario: {} ({} turns)", scenario.name, total_turns),
            None,
        );
    }

    for (index, turn) in scenario.turns.iter().enumerate() {
        // Check cancellation
        if cancel.load(Ordering::SeqCst) {
            tracing::info!("Scenario execution cancelled at turn {}", index);
            update_execution_status(
                &executions_arc,
                &session_id,
                ScenarioExecStatus::Aborted,
            );
            let mut logger = test_logger_arc.lock();
            logger.log_event(&session_id, "warn", "Scenario execution aborted", None);
            logger.end_session(&session_id, "aborted");
            return Ok(());
        }

        // Handle pause
        if pause.load(Ordering::SeqCst) {
            update_execution_status(
                &executions_arc,
                &session_id,
                ScenarioExecStatus::Paused,
            );
            {
                let mut logger = test_logger_arc.lock();
                logger.log_event(
                    &session_id,
                    "info",
                    &format!("Scenario paused at turn {}", index + 1),
                    None,
                );
            }
        }

        // Wait while paused
        while pause.load(Ordering::SeqCst) {
            if cancel.load(Ordering::SeqCst) {
                update_execution_status(
                    &executions_arc,
                    &session_id,
                    ScenarioExecStatus::Aborted,
                );
                let mut logger = test_logger_arc.lock();
                logger.log_event(&session_id, "warn", "Scenario aborted while paused", None);
                logger.end_session(&session_id, "aborted");
                return Ok(());
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }

        // Mark as running again after potential pause
        update_execution_status(
            &executions_arc,
            &session_id,
            ScenarioExecStatus::Running,
        );

        // Extract the file name for display
        let file_name = Path::new(&turn.audio_file)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| turn.audio_file.clone());

        // Update current turn info
        {
            let mut execs = executions_arc.lock();
            if let Some(exec) = execs.get_mut(&session_id) {
                exec.current_turn = index;
                exec.current_file_name = Some(file_name.clone());
            }
        }

        tracing::info!(
            "Starting turn {}/{}: {}",
            index + 1,
            total_turns,
            file_name
        );

        {
            let mut logger = test_logger_arc.lock();
            logger.log_event(
                &session_id,
                "info",
                &format!("Starting turn {}/{}: {}", index + 1, total_turns, file_name),
                Some(serde_json::json!({
                    "turn_index": index,
                    "audio_file": turn.audio_file,
                    "delay_before_ms": turn.delay_before_ms,
                    "delay_after_ms": turn.delay_after_ms,
                    "expected_transcript": turn.expected_transcript,
                    "expected_response": turn.expected_response,
                })),
            );
        }

        // Apply delay_before_ms
        if turn.delay_before_ms > 0.0 {
            let delay = std::time::Duration::from_millis(turn.delay_before_ms as u64);
            tracing::debug!("Waiting {:.0}ms before turn {}", turn.delay_before_ms, index + 1);
            if sleep_with_cancel_and_pause(delay, &cancel, &pause).await {
                // Cancelled during delay
                update_execution_status(
                    &executions_arc,
                    &session_id,
                    ScenarioExecStatus::Aborted,
                );
                let mut logger = test_logger_arc.lock();
                logger.log_event(&session_id, "warn", "Scenario aborted during pre-delay", None);
                logger.end_session(&session_id, "aborted");
                return Ok(());
            }
        }

        // Load and play the audio file.
        // Decode it on-the-fly from the resolved path.
        let audio_path = Path::new(&turn.audio_file);

        tracing::info!("Decoding audio file for scenario: {}", audio_path.display());
        let decoded = match decoder::decode_file(audio_path) {
            Ok(d) => d,
            Err(e) => {
                let err_msg = format!(
                    "Failed to decode audio file {}: {}",
                    audio_path.display(),
                    e
                );
                tracing::error!("{}", err_msg);
                {
                    let mut logger = test_logger_arc.lock();
                    logger.log_event(&session_id, "error", &err_msg, None);
                }
                update_execution_status(
                    &executions_arc,
                    &session_id,
                    ScenarioExecStatus::Failed(err_msg.clone()),
                );
                {
                    let mut logger = test_logger_arc.lock();
                    logger.end_session(&session_id, "failed");
                }
                return Err(err_msg);
            }
        };

        let duration = decoded.metadata.duration_sec;

        // Load into player and play
        {
            let player_lock = player_arc.lock();
            let player = player_lock.as_ref().ok_or_else(|| {
                let msg = "Audio player not available".to_string();
                tracing::error!("{}", msg);
                msg
            })?;

            player.load(decoded).map_err(|e| {
                let msg = format!("Failed to load audio: {}", e);
                tracing::error!("{}", msg);
                msg
            })?;

            // Update playback state
            {
                let mut ps = playback_state_arc.lock();
                ps.file_id = Some(turn.id.clone());
                ps.total_duration_sec = duration;
                ps.current_position_sec = 0.0;
                ps.is_playing = true;
                ps.is_paused = false;
            }

            player.send_command(PlayerCommand::Play);
        }

        // Wait for playback to complete
        loop {
            // Check cancellation -- lock/unlock player in its own block
            if cancel.load(Ordering::SeqCst) {
                {
                    let player_lock = player_arc.lock();
                    if let Some(ref player) = *player_lock {
                        player.send_command(PlayerCommand::Stop);
                    }
                }
                update_execution_status(
                    &executions_arc,
                    &session_id,
                    ScenarioExecStatus::Aborted,
                );
                {
                    let mut logger = test_logger_arc.lock();
                    logger.log_event(&session_id, "warn", "Scenario aborted during playback", None);
                    logger.end_session(&session_id, "aborted");
                }
                return Ok(());
            }

            // Handle pause during playback
            if pause.load(Ordering::SeqCst) {
                // Pause the player (lock scoped)
                {
                    let player_lock = player_arc.lock();
                    if let Some(ref player) = *player_lock {
                        player.send_command(PlayerCommand::Pause);
                    }
                }
                update_execution_status(
                    &executions_arc,
                    &session_id,
                    ScenarioExecStatus::Paused,
                );

                // Wait while paused -- no locks held across await
                while pause.load(Ordering::SeqCst) {
                    if cancel.load(Ordering::SeqCst) {
                        {
                            let player_lock = player_arc.lock();
                            if let Some(ref player) = *player_lock {
                                player.send_command(PlayerCommand::Stop);
                            }
                        }
                        update_execution_status(
                            &executions_arc,
                            &session_id,
                            ScenarioExecStatus::Aborted,
                        );
                        {
                            let mut logger = test_logger_arc.lock();
                            logger.log_event(
                                &session_id,
                                "warn",
                                "Scenario aborted while paused during playback",
                                None,
                            );
                            logger.end_session(&session_id, "aborted");
                        }
                        return Ok(());
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                }

                // Resume playback (lock scoped)
                {
                    let player_lock = player_arc.lock();
                    if let Some(ref player) = *player_lock {
                        player.send_command(PlayerCommand::Play);
                    }
                }
                update_execution_status(
                    &executions_arc,
                    &session_id,
                    ScenarioExecStatus::Running,
                );
            }

            let is_done = {
                let player_lock = player_arc.lock();
                if let Some(ref player) = *player_lock {
                    let status = player.get_status();
                    !status.is_playing && !status.is_paused
                } else {
                    true
                }
            };

            if is_done {
                break;
            }

            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }

        {
            let mut logger = test_logger_arc.lock();
            logger.log_event(
                &session_id,
                "info",
                &format!("Turn {}/{} completed: {}", index + 1, total_turns, file_name),
                None,
            );
        }

        tracing::info!("Turn {}/{} completed: {}", index + 1, total_turns, file_name);

        // Apply delay_after_ms
        if turn.delay_after_ms > 0.0 {
            let delay = std::time::Duration::from_millis(turn.delay_after_ms as u64);
            tracing::debug!("Waiting {:.0}ms after turn {}", turn.delay_after_ms, index + 1);
            if sleep_with_cancel_and_pause(delay, &cancel, &pause).await {
                update_execution_status(
                    &executions_arc,
                    &session_id,
                    ScenarioExecStatus::Aborted,
                );
                let mut logger = test_logger_arc.lock();
                logger.log_event(&session_id, "warn", "Scenario aborted during post-delay", None);
                logger.end_session(&session_id, "aborted");
                return Ok(());
            }
        }
    }

    // Scenario completed successfully
    tracing::info!("Scenario execution completed: {}", scenario.name);
    update_execution_status(
        &executions_arc,
        &session_id,
        ScenarioExecStatus::Completed,
    );

    {
        let mut logger = test_logger_arc.lock();
        logger.log_event(
            &session_id,
            "info",
            &format!("Scenario completed: {} ({} turns)", scenario.name, total_turns),
            None,
        );
        logger.end_session(&session_id, "completed");
    }

    Ok(())
}

/// Update the execution status of a scenario in the shared state
fn update_execution_status(
    executions_arc: &Arc<Mutex<HashMap<String, ScenarioExecutionState>>>,
    session_id: &str,
    status: ScenarioExecStatus,
) {
    let mut execs = executions_arc.lock();
    if let Some(exec) = execs.get_mut(session_id) {
        exec.status = status;
    }
}

/// Sleep for a duration, checking for both cancellation and pause every 100ms.
/// Returns true if cancelled (not paused -- pause is handled by the caller).
async fn sleep_with_cancel_and_pause(
    duration: std::time::Duration,
    cancel: &AtomicBool,
    pause: &AtomicBool,
) -> bool {
    let interval = std::time::Duration::from_millis(100);
    let mut remaining = duration;

    while remaining > std::time::Duration::ZERO {
        if cancel.load(Ordering::SeqCst) {
            return true;
        }

        // If paused, wait without consuming remaining time
        while pause.load(Ordering::SeqCst) {
            if cancel.load(Ordering::SeqCst) {
                return true;
            }
            tokio::time::sleep(interval).await;
        }

        let sleep_time = remaining.min(interval);
        tokio::time::sleep(sleep_time).await;
        remaining = remaining.saturating_sub(sleep_time);
    }

    false
}
