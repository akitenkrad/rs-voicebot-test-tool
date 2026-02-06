//! TTS test execution runner.
//!
//! Handles the sequential execution of TTS test cases with recording support.

use std::collections::HashMap;
use std::io::Cursor;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use chrono::Utc;
use parking_lot::Mutex;
use tauri::{AppHandle, Emitter};

use super::config::TtsConfig;
use super::csv_parser::TtsTestCase;
use super::generator;
use super::types::{TtsTestResult, TtsTestSession, TtsTestStatus};
use crate::audio::decoder::{AudioMetadata, DecodedAudio};
use crate::audio::player::{AudioPlayer, PlayerCommand};
use crate::audio::recorder::AudioRecorder;
use crate::audio::RecordingConfig;

/// Debug-only logging macro. Compiled out entirely in release builds.
macro_rules! dbg_log {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        tracing::debug!($($arg)*)
    };
}

/// TTS test runner that manages the execution of a test session.
pub struct TtsTestRunner {
    /// The test session being executed
    session: Arc<Mutex<TtsTestSession>>,
    /// Flag to signal cancellation
    cancel_flag: Arc<AtomicBool>,
    /// Flag to signal pause
    pause_flag: Arc<AtomicBool>,
}

impl TtsTestRunner {
    /// Create a new TTS test runner for the given session.
    pub fn new(session: TtsTestSession) -> Self {
        Self {
            session: Arc::new(Mutex::new(session)),
            cancel_flag: Arc::new(AtomicBool::new(false)),
            pause_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Get a clone of the cancel flag for external control.
    pub fn cancel_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.cancel_flag)
    }

    /// Get a clone of the pause flag for external control.
    pub fn pause_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.pause_flag)
    }

    /// Get a snapshot of the current session state.
    pub fn get_session(&self) -> TtsTestSession {
        self.session.lock().clone()
    }

    /// Update the session status.
    pub fn update_status(&self, status: TtsTestStatus) {
        let mut session = self.session.lock();
        session.status = status;
    }

    /// Add a result to the session.
    pub fn add_result(&self, result: TtsTestResult) {
        let mut session = self.session.lock();
        session.results.push(result);
    }

    /// Finalize the session by setting the end time.
    pub fn finalize_session(&self) {
        let mut session = self.session.lock();
        session.ended_at = Some(Utc::now());
    }
}

// ---------------------------------------------------------------------------
// Recording thread infrastructure
// ---------------------------------------------------------------------------

/// Result returned from the recording thread.
enum RecordingResult {
    /// Successfully recorded audio samples.
    Samples {
        data: Vec<f32>,
        sample_rate: u32,
        channels: u16,
    },
    /// Recording failed with an error.
    Error(String),
}

/// Handle for controlling a recording running on a dedicated OS thread.
struct RecordingHandle {
    /// Shared flag: set to `true` by the cpal callback when silence is detected.
    silence_detected: Arc<AtomicBool>,
    /// Send `()` to signal the recording thread to stop.
    stop_tx: crossbeam_channel::Sender<()>,
    /// Receive the recorded samples (or an error) after stopping.
    result_rx: crossbeam_channel::Receiver<RecordingResult>,
}

/// Spawn a dedicated OS thread that owns `AudioRecorder` + `cpal::Stream`.
///
/// Returns a [`RecordingHandle`] that the async side can use to monitor
/// silence detection and to stop recording.
fn start_recording_thread(config: RecordingConfig) -> Result<RecordingHandle, String> {
    dbg_log!(
        "start_recording_thread: device={:?}, sample_rate={}, channels={}, silence_threshold={}dB, silence_duration={}ms, max_sec={}",
        config.device_name, config.sample_rate, config.channels,
        config.silence_threshold_db, config.silence_duration_ms, config.max_recording_sec
    );

    let recorder = AudioRecorder::new(config.clone());

    // Grab a clone of the silence flag *before* moving the recorder into the thread.
    let silence_detected = recorder.silence_detected_flag();

    let (stop_tx, stop_rx) = crossbeam_channel::bounded::<()>(1);
    let (result_tx, result_rx) = crossbeam_channel::bounded::<RecordingResult>(1);

    let sample_rate = config.sample_rate;
    let channels = config.channels;

    std::thread::Builder::new()
        .name("tts-recorder".into())
        .spawn(move || {
            dbg_log!("[rec-thread] spawned, calling start_recording()");

            let mut recorder = recorder;

            let stream = match recorder.start_recording() {
                Ok(s) => {
                    dbg_log!("[rec-thread] start_recording() succeeded, cpal stream active");
                    s
                }
                Err(e) => {
                    dbg_log!("[rec-thread] start_recording() FAILED: {}", e);
                    let _ = result_tx.send(RecordingResult::Error(e));
                    return;
                }
            };

            tracing::info!("Recording thread started");
            dbg_log!("[rec-thread] blocking on stop_rx.recv()");

            // Block until the async side sends a stop signal.
            #[cfg(debug_assertions)]
            let recv_result = stop_rx.recv();
            #[cfg(not(debug_assertions))]
            let _ = stop_rx.recv();
            dbg_log!("[rec-thread] stop_rx.recv() returned: {:?}", recv_result.is_ok());

            // Stop recording and collect samples.
            dbg_log!("[rec-thread] calling stop_recording()");
            let samples = recorder.stop_recording();
            dbg_log!("[rec-thread] stop_recording() done, {} samples", samples.len());

            dbg_log!("[rec-thread] dropping cpal stream");
            drop(stream); // explicitly drop in this thread
            dbg_log!("[rec-thread] cpal stream dropped");

            tracing::info!("Recording thread stopped, captured {} samples", samples.len());

            #[cfg(debug_assertions)]
            let send_result = result_tx.send(RecordingResult::Samples {
                data: samples,
                sample_rate,
                channels,
            });
            #[cfg(not(debug_assertions))]
            let _ = result_tx.send(RecordingResult::Samples {
                data: samples,
                sample_rate,
                channels,
            });
            dbg_log!("[rec-thread] result_tx.send() ok={}", send_result.is_ok());
        })
        .map_err(|e| format!("Failed to spawn recording thread: {}", e))?;

    dbg_log!("start_recording_thread: thread spawned successfully");

    Ok(RecordingHandle {
        silence_detected,
        stop_tx,
        result_rx,
    })
}

// ---------------------------------------------------------------------------

/// Execute TTS test as a standalone async function.
///
/// This function is designed to be spawned in a tokio task without holding
/// any non-Send types across await points.
pub async fn execute_tts_test(
    session_id: String,
    test_cases: Vec<TtsTestCase>,
    tts_config: TtsConfig,
    recording_config: RecordingConfig,
    cancel_flag: Arc<AtomicBool>,
    pause_flag: Arc<AtomicBool>,
    player_arc: Arc<Mutex<Option<AudioPlayer>>>,
    tts_sessions_arc: Arc<Mutex<HashMap<String, TtsTestRunner>>>,
    app_handle: Option<AppHandle>,
) -> Result<(), String> {
    let total_cases = test_cases.len();

    tracing::info!(
        "Starting TTS test execution: {} test cases",
        total_cases
    );

    // Update status to running
    update_runner_status(&tts_sessions_arc, &session_id, TtsTestStatus::Running { current_index: 0 });
    emit_status_update(&tts_sessions_arc, &session_id, &app_handle);

    for (index, test_case) in test_cases.iter().enumerate() {
        // Check for cancellation
        if cancel_flag.load(Ordering::SeqCst) {
            tracing::info!("TTS test execution cancelled at index {}", index);
            update_runner_status(&tts_sessions_arc, &session_id, TtsTestStatus::Aborted);
            finalize_runner_session(&tts_sessions_arc, &session_id);
            emit_status_update(&tts_sessions_arc, &session_id, &app_handle);
            return Ok(());
        }

        // Handle pause
        if pause_flag.load(Ordering::SeqCst) {
            update_runner_status(&tts_sessions_arc, &session_id, TtsTestStatus::Paused { current_index: index });
            emit_status_update(&tts_sessions_arc, &session_id, &app_handle);
            tracing::info!("TTS test execution paused at index {}", index);
        }

        // Wait while paused
        while pause_flag.load(Ordering::SeqCst) {
            if cancel_flag.load(Ordering::SeqCst) {
                update_runner_status(&tts_sessions_arc, &session_id, TtsTestStatus::Aborted);
                finalize_runner_session(&tts_sessions_arc, &session_id);
                emit_status_update(&tts_sessions_arc, &session_id, &app_handle);
                return Ok(());
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }

        // Update status to running this test case
        update_runner_status(&tts_sessions_arc, &session_id, TtsTestStatus::Running { current_index: index });
        emit_status_update(&tts_sessions_arc, &session_id, &app_handle);

        tracing::info!(
            "Executing test case {}/{}: {}",
            index + 1,
            total_cases,
            test_case.id
        );

        // Execute the single test case
        let result = execute_single_test_case(
            test_case,
            &tts_config,
            &recording_config,
            &cancel_flag,
            &pause_flag,
            &player_arc,
        )
        .await;

        // Store the result
        add_runner_result(&tts_sessions_arc, &session_id, result);
        emit_status_update(&tts_sessions_arc, &session_id, &app_handle);
    }

    // Mark session as completed
    dbg_log!("[session:{}] all {} test cases processed, setting status=Completed", session_id, total_cases);
    tracing::info!("TTS test execution completed: {} test cases", total_cases);
    update_runner_status(&tts_sessions_arc, &session_id, TtsTestStatus::Completed);
    finalize_runner_session(&tts_sessions_arc, &session_id);
    emit_status_update(&tts_sessions_arc, &session_id, &app_handle);
    dbg_log!("[session:{}] status=Completed emitted, execute_tts_test returning Ok", session_id);

    Ok(())
}

/// Execute a single test case with recording support.
///
/// Flow:
/// 1. Generate TTS audio from text
/// 2. Start recording on a dedicated OS thread (captures chatbot response)
/// 3. Play TTS audio through the configured output device
/// 4. Wait for playback to complete
/// 5. Wait for silence detection (chatbot finished responding)
/// 6. Stop recording and collect response audio
async fn execute_single_test_case(
    test_case: &TtsTestCase,
    tts_config: &TtsConfig,
    recording_config: &RecordingConfig,
    cancel_flag: &Arc<AtomicBool>,
    pause_flag: &Arc<AtomicBool>,
    player_arc: &Arc<Mutex<Option<AudioPlayer>>>,
) -> TtsTestResult {
    #[cfg(debug_assertions)]
    let case_timer = std::time::Instant::now();
    let started_at = Utc::now();
    dbg_log!("[case:{}] === execute_single_test_case START ===", test_case.id);

    // --- Step 1: Generate TTS audio ---
    tracing::info!(
        "Generating TTS for test case {}: {}",
        test_case.id,
        &test_case.text[..test_case.text.len().min(50)]
    );

    let tts_bytes = match generator::generate_tts(&test_case.text, tts_config).await {
        Ok(bytes) => bytes,
        Err(e) => {
            let error_msg = format!("TTS generation failed: {}", e);
            tracing::error!("{}", error_msg);
            return TtsTestResult {
                test_case_id: test_case.id.clone(),
                input_text: test_case.text.clone(),
                tts_duration_sec: 0.0,
                response_duration_sec: None,
                started_at,
                ended_at: Utc::now(),
                error: Some(error_msg),
                tts_audio: Vec::new(),
                response_audio: None,
            };
        }
    };

    tracing::info!(
        "Generated {} bytes of TTS audio for test case {}",
        tts_bytes.len(),
        test_case.id
    );

    // --- Step 2: Decode TTS bytes ---
    let decoded_audio = match decode_wav_bytes(&tts_bytes) {
        Ok(audio) => audio,
        Err(e) => {
            let error_msg = format!("Failed to decode TTS audio: {}", e);
            tracing::error!("{}", error_msg);
            return TtsTestResult {
                test_case_id: test_case.id.clone(),
                input_text: test_case.text.clone(),
                tts_duration_sec: 0.0,
                response_duration_sec: None,
                started_at,
                ended_at: Utc::now(),
                error: Some(error_msg),
                tts_audio: tts_bytes,
                response_audio: None,
            };
        }
    };

    let tts_duration_sec = decoded_audio.metadata.duration_sec;

    // --- Step 3: Start recording (dedicated OS thread) ---
    dbg_log!("[case:{}] Step 3: starting recording thread (elapsed: {:.1}ms)", test_case.id, case_timer.elapsed().as_secs_f64() * 1000.0);
    let recording_handle = match start_recording_thread(recording_config.clone()) {
        Ok(handle) => {
            tracing::info!("Recording started for test case {}", test_case.id);
            dbg_log!("[case:{}] recording_handle=Some, silence_detected={}", test_case.id, handle.silence_detected.load(Ordering::SeqCst));
            Some(handle)
        }
        Err(e) => {
            tracing::warn!("Failed to start recording for test case {}: {}", test_case.id, e);
            dbg_log!("[case:{}] recording_handle=None (start failed)", test_case.id);
            None
        }
    };

    // --- Step 4: Load and play TTS audio ---
    dbg_log!("[case:{}] Step 4: loading and playing audio (elapsed: {:.1}ms)", test_case.id, case_timer.elapsed().as_secs_f64() * 1000.0);
    {
        let player_lock = player_arc.lock();
        if let Some(ref player) = *player_lock {
            if let Err(e) = player.load(decoded_audio) {
                let error_msg = format!("Failed to load audio: {}", e);
                tracing::error!("{}", error_msg);
                // Stop recording if started
                if let Some(handle) = recording_handle {
                    let _ = handle.stop_tx.send(());
                }
                return TtsTestResult {
                    test_case_id: test_case.id.clone(),
                    input_text: test_case.text.clone(),
                    tts_duration_sec,
                    response_duration_sec: None,
                    started_at,
                    ended_at: Utc::now(),
                    error: Some(error_msg),
                    tts_audio: tts_bytes,
                    response_audio: None,
                };
            }
            player.send_command(PlayerCommand::Play);
        } else {
            let error_msg = "Audio player not available".to_string();
            tracing::error!("{}", error_msg);
            if let Some(handle) = recording_handle {
                let _ = handle.stop_tx.send(());
            }
            return TtsTestResult {
                test_case_id: test_case.id.clone(),
                input_text: test_case.text.clone(),
                tts_duration_sec,
                response_duration_sec: None,
                started_at,
                ended_at: Utc::now(),
                error: Some(error_msg),
                tts_audio: tts_bytes,
                response_audio: None,
            };
        }
    }

    tracing::info!("Started playback for test case {}", test_case.id);

    // --- Step 5: Wait for playback to complete ---
    dbg_log!("[case:{}] Step 5: waiting for playback completion (elapsed: {:.1}ms)", test_case.id, case_timer.elapsed().as_secs_f64() * 1000.0);
    #[cfg(debug_assertions)]
    let playback_wait_start = std::time::Instant::now();
    #[cfg(debug_assertions)]
    let mut playback_poll_count: u64 = 0;
    loop {
        #[cfg(debug_assertions)]
        {
            playback_poll_count += 1;
            // Log every 2 seconds (40 iterations * 50ms)
            if playback_poll_count % 40 == 0 {
                let player_status = {
                    let player_lock = player_arc.lock();
                    player_lock.as_ref().map(|p| {
                        let s = p.get_status();
                        format!("is_playing={}, is_paused={}, pos={:.2}s, dur={:.2}s", s.is_playing, s.is_paused, s.position_sec, s.duration_sec)
                    })
                };
                dbg_log!(
                    "[case:{}] Step 5 poll #{}: player_status={:?}, elapsed={:.1}s",
                    test_case.id, playback_poll_count, player_status,
                    playback_wait_start.elapsed().as_secs_f64()
                );
            }
        }
        if cancel_flag.load(Ordering::SeqCst) {
            {
                let player_lock = player_arc.lock();
                if let Some(ref player) = *player_lock {
                    player.send_command(PlayerCommand::Stop);
                }
            }
            if let Some(handle) = recording_handle {
                let _ = handle.stop_tx.send(());
            }
            return TtsTestResult {
                test_case_id: test_case.id.clone(),
                input_text: test_case.text.clone(),
                tts_duration_sec,
                response_duration_sec: None,
                started_at,
                ended_at: Utc::now(),
                error: Some("Cancelled".to_string()),
                tts_audio: tts_bytes.clone(),
                response_audio: None,
            };
        }

        if pause_flag.load(Ordering::SeqCst) {
            {
                let player_lock = player_arc.lock();
                if let Some(ref player) = *player_lock {
                    player.send_command(PlayerCommand::Pause);
                }
            }

            while pause_flag.load(Ordering::SeqCst) {
                if cancel_flag.load(Ordering::SeqCst) {
                    {
                        let player_lock = player_arc.lock();
                        if let Some(ref player) = *player_lock {
                            player.send_command(PlayerCommand::Stop);
                        }
                    }
                    if let Some(handle) = recording_handle {
                        let _ = handle.stop_tx.send(());
                    }
                    return TtsTestResult {
                        test_case_id: test_case.id.clone(),
                        input_text: test_case.text.clone(),
                        tts_duration_sec,
                        response_duration_sec: None,
                        started_at,
                        ended_at: Utc::now(),
                        error: Some("Cancelled".to_string()),
                        tts_audio: tts_bytes.clone(),
                        response_audio: None,
                    };
                }
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }

            {
                let player_lock = player_arc.lock();
                if let Some(ref player) = *player_lock {
                    player.send_command(PlayerCommand::Play);
                }
            }
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
            #[cfg(debug_assertions)]
            tracing::debug!(
                "[case:{}] Step 5: playback done after {:.2}s ({} polls)",
                test_case.id, playback_wait_start.elapsed().as_secs_f64(), playback_poll_count
            );
            break;
        }

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }

    tracing::info!("Playback completed for test case {}", test_case.id);

    // --- Step 6: Wait for silence detection (chatbot response finished) ---
    //
    // Use max_recording_sec as the overall timeout for the silence detection
    // wait. This is the maximum time we'll wait after playback for the chatbot
    // to respond and then go silent.
    let (response_duration_sec, response_audio) = if let Some(handle) = recording_handle {
        let recording_start = std::time::Instant::now();
        let max_wait = std::time::Duration::from_secs(recording_config.max_recording_sec);
        dbg_log!(
            "[case:{}] Step 6: silence detection wait (max_wait={}s, silence_detected={}, total_elapsed={:.1}ms)",
            test_case.id, recording_config.max_recording_sec,
            handle.silence_detected.load(Ordering::SeqCst),
            case_timer.elapsed().as_secs_f64() * 1000.0
        );

        // Poll silence_detected flag until silence is found or timeout.
        // If silence was already detected during playback (e.g., quiet input
        // device), the loop breaks immediately.
        let mut timed_out = false;
        #[cfg(debug_assertions)]
        let mut silence_poll_count: u64 = 0;
        loop {
            #[cfg(debug_assertions)]
            {
                silence_poll_count += 1;
                // Log every 5 seconds (50 iterations * 100ms)
                if silence_poll_count % 50 == 0 {
                    dbg_log!(
                        "[case:{}] Step 6 poll #{}: silence_detected={}, elapsed={:.1}s / max={}s",
                        test_case.id, silence_poll_count,
                        handle.silence_detected.load(Ordering::SeqCst),
                        recording_start.elapsed().as_secs_f64(),
                        recording_config.max_recording_sec
                    );
                }
            }
            if cancel_flag.load(Ordering::SeqCst) {
                let _ = handle.stop_tx.send(());
                return TtsTestResult {
                    test_case_id: test_case.id.clone(),
                    input_text: test_case.text.clone(),
                    tts_duration_sec,
                    response_duration_sec: None,
                    started_at,
                    ended_at: Utc::now(),
                    error: Some("Cancelled".to_string()),
                    tts_audio: tts_bytes.clone(),
                    response_audio: None,
                };
            }

            if handle.silence_detected.load(Ordering::SeqCst) {
                tracing::info!(
                    "Silence detected for test case {} after {:.1}s",
                    test_case.id,
                    recording_start.elapsed().as_secs_f64()
                );
                break;
            }

            if recording_start.elapsed() >= max_wait {
                timed_out = true;
                tracing::warn!(
                    "Recording timeout for test case {} after {}s",
                    test_case.id,
                    recording_config.max_recording_sec
                );
                break;
            }

            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }

        let response_elapsed = recording_start.elapsed().as_secs_f64();
        if timed_out {
            tracing::info!(
                "Recording timed out for test case {}, elapsed: {:.1}s",
                test_case.id,
                response_elapsed
            );
        }

        // Signal the recording thread to stop
        dbg_log!("[case:{}] Step 6: sending stop signal to recording thread", test_case.id);
        #[cfg(debug_assertions)]
        let stop_send_result = handle.stop_tx.send(());
        #[cfg(not(debug_assertions))]
        let _ = handle.stop_tx.send(());
        dbg_log!("[case:{}] Step 6: stop_tx.send() ok={}", test_case.id, stop_send_result.is_ok());

        // Collect recorded samples with a bounded timeout to prevent
        // indefinite blocking if the recording thread hangs (e.g., during
        // cpal stream cleanup on certain platforms).
        const RECV_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);
        dbg_log!("[case:{}] Step 6: waiting for result_rx (timeout={}s)", test_case.id, RECV_TIMEOUT.as_secs());

        match handle.result_rx.recv_timeout(RECV_TIMEOUT) {
            Ok(RecordingResult::Samples { data, sample_rate, channels }) => {
                if data.is_empty() {
                    tracing::info!("No audio recorded for test case {}", test_case.id);
                    (Some(response_elapsed), None)
                } else {
                    tracing::info!(
                        "Recorded {} samples for test case {}",
                        data.len(),
                        test_case.id
                    );
                    match AudioRecorder::samples_to_wav_bytes(&data, sample_rate, channels) {
                        Ok(wav_bytes) => (Some(response_elapsed), Some(wav_bytes)),
                        Err(e) => {
                            tracing::warn!("Failed to encode response WAV: {}", e);
                            (Some(response_elapsed), None)
                        }
                    }
                }
            }
            Ok(RecordingResult::Error(e)) => {
                tracing::warn!("Recording error for test case {}: {}", test_case.id, e);
                (Some(response_elapsed), None)
            }
            Err(e) => {
                tracing::warn!("Failed to receive recording result (timeout or disconnected): {}", e);
                (Some(response_elapsed), None)
            }
        }
    } else {
        // No recording handle — recording was not available
        (None, None)
    };

    dbg_log!(
        "[case:{}] === execute_single_test_case END === total={:.2}s, tts_dur={:.2}s, resp_dur={:?}s, has_resp_audio={}",
        test_case.id, case_timer.elapsed().as_secs_f64(),
        tts_duration_sec, response_duration_sec, response_audio.is_some()
    );

    TtsTestResult {
        test_case_id: test_case.id.clone(),
        input_text: test_case.text.clone(),
        tts_duration_sec,
        response_duration_sec,
        started_at,
        ended_at: Utc::now(),
        error: None,
        tts_audio: tts_bytes,
        response_audio,
    }
}

/// Update runner status from the sessions map.
fn update_runner_status(
    tts_sessions_arc: &Arc<Mutex<HashMap<String, TtsTestRunner>>>,
    session_id: &str,
    status: TtsTestStatus,
) {
    let sessions = tts_sessions_arc.lock();
    if let Some(runner) = sessions.get(session_id) {
        runner.update_status(status);
    }
}

/// Add result to runner from the sessions map.
fn add_runner_result(
    tts_sessions_arc: &Arc<Mutex<HashMap<String, TtsTestRunner>>>,
    session_id: &str,
    result: TtsTestResult,
) {
    let sessions = tts_sessions_arc.lock();
    if let Some(runner) = sessions.get(session_id) {
        runner.add_result(result);
    }
}

/// Finalize runner session from the sessions map.
fn finalize_runner_session(
    tts_sessions_arc: &Arc<Mutex<HashMap<String, TtsTestRunner>>>,
    session_id: &str,
) {
    let sessions = tts_sessions_arc.lock();
    if let Some(runner) = sessions.get(session_id) {
        runner.finalize_session();
    }
}

/// Emit status update from the sessions map.
fn emit_status_update(
    tts_sessions_arc: &Arc<Mutex<HashMap<String, TtsTestRunner>>>,
    session_id: &str,
    app_handle: &Option<AppHandle>,
) {
    if let Some(handle) = app_handle {
        let session = {
            let sessions = tts_sessions_arc.lock();
            sessions.get(session_id).map(|r| r.get_session())
        };
        if let Some(session) = session {
            if let Err(e) = handle.emit("tts-test-status-update", &session) {
                tracing::warn!("Failed to emit TTS test status update: {}", e);
            }
        }
    }
}

/// Decode audio bytes into DecodedAudio using symphonia (supports WAV, MP3, etc.).
fn decode_wav_bytes(bytes: &[u8]) -> Result<DecodedAudio, String> {
    use symphonia::core::audio::SampleBuffer;
    use symphonia::core::codecs::DecoderOptions;
    use symphonia::core::formats::FormatOptions;
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::probe::Hint;

    let cursor = Cursor::new(bytes.to_vec());
    let mss = MediaSourceStream::new(Box::new(cursor), Default::default());

    let hint = Hint::new();
    let format_opts = FormatOptions::default();
    let metadata_opts = MetadataOptions::default();

    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &format_opts, &metadata_opts)
        .map_err(|e| format!("Failed to probe audio format: {}", e))?;

    let mut format = probed.format;

    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
        .ok_or("No supported audio track found")?;

    let track_id = track.id;
    let codec_params = track.codec_params.clone();

    let sample_rate = codec_params.sample_rate.ok_or("Unknown sample rate")?;
    let channels = codec_params
        .channels
        .map(|c| c.count() as u16)
        .ok_or("Unknown channel count")?;

    let decoder_opts = DecoderOptions::default();
    let mut decoder = symphonia::default::get_codecs()
        .make(&codec_params, &decoder_opts)
        .map_err(|e| format!("Failed to create decoder: {}", e))?;

    let mut samples = Vec::new();

    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(symphonia::core::errors::Error::IoError(ref e))
                if e.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                break;
            }
            Err(e) => return Err(format!("Failed to read packet: {}", e)),
        };

        if packet.track_id() != track_id {
            continue;
        }

        let decoded = match decoder.decode(&packet) {
            Ok(decoded) => decoded,
            Err(e) => {
                tracing::warn!("Decode error (skipping packet): {}", e);
                continue;
            }
        };

        let spec = *decoded.spec();
        let mut sample_buf = SampleBuffer::<f32>::new(decoded.capacity() as u64, spec);
        sample_buf.copy_interleaved_ref(decoded);
        samples.extend_from_slice(sample_buf.samples());
    }

    let total_samples = samples.len() as u64 / channels as u64;
    let duration_sec = total_samples as f64 / sample_rate as f64;

    Ok(DecodedAudio {
        metadata: AudioMetadata {
            sample_rate,
            channels,
            duration_sec,
            format: "audio".to_string(),
            total_samples,
        },
        samples,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_as_str() {
        assert_eq!(TtsTestStatus::Pending.as_str(), "pending");
        assert_eq!(TtsTestStatus::Running { current_index: 0 }.as_str(), "running");
        assert_eq!(TtsTestStatus::Paused { current_index: 0 }.as_str(), "paused");
        assert_eq!(TtsTestStatus::Completed.as_str(), "completed");
        assert_eq!(TtsTestStatus::Aborted.as_str(), "aborted");
        assert_eq!(
            TtsTestStatus::Failed { message: "error".to_string() }.as_str(),
            "failed"
        );
    }
}
