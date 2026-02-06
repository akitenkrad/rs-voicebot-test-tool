use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use crate::audio::decoder::DecodedAudio;
use crate::audio::player::AudioPlayer;
use crate::audio::recorder::AudioRecorder;
use crate::audio::recording::RecordingConfig;
use crate::config::AppConfig;
use crate::device::{self, VirtualDeviceManager};
use crate::logging::reporter::TestLogger;
use crate::scenario::executor::ScenarioExecutionState;
use crate::tts::TtsTestRunner;
use crate::types::{
    AudioFileInfo, DeviceInfo, PlaybackState, PlaylistInfo, ScenarioInfo,
};

/// Internal state for tracking playlist playback (not serializable)
pub struct PlaylistPlaybackState {
    pub playlist_id: String,
    pub current_item_index: usize,
    pub total_items: usize,
    pub is_playing: bool,
    pub loop_mode: bool,
    pub cancel: Arc<AtomicBool>,
}

/// Application state managed by Tauri
pub struct AppState {
    /// Application configuration
    pub config: Arc<Mutex<AppConfig>>,
    /// Loaded audio files
    pub audio_files: Arc<Mutex<Vec<AudioFileInfo>>>,
    /// Current playback state
    pub playback_state: Arc<Mutex<PlaybackState>>,
    /// Active virtual device info
    pub active_device: Arc<Mutex<Option<DeviceInfo>>>,
    /// Platform-specific virtual device manager
    pub device_manager: Arc<Mutex<Box<dyn VirtualDeviceManager>>>,
    /// Loaded scenarios
    pub scenarios: Arc<Mutex<Vec<ScenarioInfo>>>,
    /// Playlists
    pub playlists: Arc<Mutex<Vec<PlaylistInfo>>>,
    /// Active test session IDs
    pub active_sessions: Arc<Mutex<Vec<String>>>,
    /// Audio player instance
    pub player: Arc<Mutex<Option<AudioPlayer>>>,
    /// Cache of decoded audio data, keyed by file ID
    pub decoded_cache: Arc<Mutex<HashMap<String, DecodedAudio>>>,
    /// Playlist playback state
    pub playlist_playback: Arc<Mutex<Option<PlaylistPlaybackState>>>,
    /// Test logger for managing test sessions and events
    pub test_logger: Arc<Mutex<TestLogger>>,
    /// Active scenario execution states, keyed by session_id
    pub scenario_executions: Arc<Mutex<HashMap<String, ScenarioExecutionState>>>,
    /// Recording configuration
    pub recording_config: Arc<Mutex<RecordingConfig>>,
    /// Audio recorder instance (active during recording)
    pub audio_recorder: Arc<Mutex<Option<AudioRecorder>>>,
    /// Active TTS test sessions, keyed by session_id
    pub tts_sessions: Arc<Mutex<HashMap<String, TtsTestRunner>>>,
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        // Create the audio player. If it fails (e.g., no audio device),
        // we continue without a player and log the error.
        let player = match AudioPlayer::new() {
            Ok(p) => {
                tracing::info!("Audio player initialized successfully");
                Some(p)
            }
            Err(e) => {
                tracing::warn!("Failed to initialize audio player: {}. Playback will not be available.", e);
                None
            }
        };

        Self {
            config: Arc::new(Mutex::new(config)),
            audio_files: Arc::new(Mutex::new(Vec::new())),
            playback_state: Arc::new(Mutex::new(PlaybackState::default())),
            active_device: Arc::new(Mutex::new(None)),
            device_manager: Arc::new(Mutex::new(device::create_device_manager())),
            scenarios: Arc::new(Mutex::new(Vec::new())),
            playlists: Arc::new(Mutex::new(Vec::new())),
            active_sessions: Arc::new(Mutex::new(Vec::new())),
            player: Arc::new(Mutex::new(player)),
            decoded_cache: Arc::new(Mutex::new(HashMap::new())),
            playlist_playback: Arc::new(Mutex::new(None)),
            test_logger: Arc::new(Mutex::new(TestLogger::new())),
            scenario_executions: Arc::new(Mutex::new(HashMap::new())),
            recording_config: Arc::new(Mutex::new(RecordingConfig::default())),
            audio_recorder: Arc::new(Mutex::new(None)),
            tts_sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}
