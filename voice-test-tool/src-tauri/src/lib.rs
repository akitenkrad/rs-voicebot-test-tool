mod audio;
mod commands;
mod config;
mod device;
mod error;
mod logging;
mod scenario;
mod state;
mod tts;
mod types;

use state::AppState;
use tracing_subscriber::EnvFilter;

/// Initialize the tracing subscriber.
///
/// - **Debug builds** (`cargo build` / `npm run tauri dev`):
///   Logs to both stdout and a rotating log file under `logs/`.
///   Default level: `debug` (overridable via `RUST_LOG`).
///   Files rotate daily; old files are kept for up to 7 days.
///
/// - **Release builds** (`cargo build --release` / `npm run tauri build`):
///   Logs to stdout only. Default level: `info`.
fn init_tracing() {
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    let default_level = if cfg!(debug_assertions) { "debug" } else { "info" };
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_level));

    let stdout_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_thread_names(true);

    #[cfg(debug_assertions)]
    {
        // In debug builds, also write to a daily-rotating log file.
        let log_dir = std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .join("logs");

        let file_appender = tracing_appender::rolling::RollingFileAppender::builder()
            .rotation(tracing_appender::rolling::Rotation::DAILY)
            .filename_prefix("voice-test-tool")
            .filename_suffix("log")
            .max_log_files(7)
            .build(&log_dir)
            .expect("failed to create log file appender");

        let file_layer = tracing_subscriber::fmt::layer()
            .with_writer(file_appender)
            .with_ansi(false)
            .with_target(true)
            .with_thread_names(true);

        tracing_subscriber::registry()
            .with(env_filter)
            .with(stdout_layer)
            .with(file_layer)
            .init();

        tracing::info!("Debug logging enabled: stdout + file ({})", log_dir.display());
    }

    #[cfg(not(debug_assertions))]
    {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(stdout_layer)
            .init();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize tracing subscriber for structured logging
    init_tracing();

    // Load application configuration
    let app_config = config::load_config();

    // Create application state
    let app_state = AppState::new(app_config);

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            // Audio file management
            commands::audio::load_audio_file,
            commands::audio::list_audio_files,
            commands::audio::remove_audio_file,
            commands::audio::get_waveform_data,
            // Virtual device management
            commands::device::create_virtual_device,
            commands::device::destroy_virtual_device,
            commands::device::list_audio_devices,
            commands::device::set_default_input_device,
            commands::device::get_device_status,
            // Playback control
            commands::playback::play,
            commands::playback::pause,
            commands::playback::stop,
            commands::playback::seek,
            commands::playback::set_playback_speed,
            commands::playback::set_volume,
            commands::playback::get_playback_state,
            commands::playback::get_audio_system_status,
            // Playlist management
            commands::playlist::create_playlist,
            commands::playlist::add_to_playlist,
            commands::playlist::play_playlist,
            commands::playlist::list_playlists,
            commands::playlist::get_playlist,
            commands::playlist::delete_playlist,
            commands::playlist::remove_from_playlist,
            commands::playlist::reorder_playlist,
            commands::playlist::update_playlist_item_silence,
            commands::playlist::get_playlist_playback_status,
            commands::playlist::stop_playlist,
            // Scenario management
            commands::scenario::load_scenario,
            commands::scenario::save_scenario,
            commands::scenario::execute_scenario,
            commands::scenario::pause_scenario,
            commands::scenario::resume_scenario,
            commands::scenario::abort_scenario,
            commands::scenario::list_scenarios,
            commands::scenario::delete_scenario,
            commands::scenario::get_scenario_execution_status,
            // Configuration
            commands::config::get_config,
            commands::config::update_config,
            // Logging and reports
            commands::log::get_session_logs,
            commands::log::export_report,
            commands::log::list_test_sessions,
            commands::log::get_test_session,
            // TTS generation
            commands::tts::load_tts_csv,
            commands::tts::preview_tts,
            // TTS test execution
            commands::tts::start_tts_test,
            commands::tts::pause_tts_test,
            commands::tts::resume_tts_test,
            commands::tts::abort_tts_test,
            commands::tts::get_tts_test_status,
            commands::tts::get_tts_test_session,
            commands::tts::export_tts_test,
            // Recording
            commands::recording::list_input_devices,
            commands::recording::get_recording_config,
            commands::recording::update_recording_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
