use serde::{Deserialize, Serialize};

/// Status of the audio subsystem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSystemStatus {
    pub available: bool,
    pub error: Option<String>,
}

/// Information about a loaded audio file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioFileInfo {
    pub id: String,
    pub name: String,
    pub path: String,
    pub sample_rate: u32,
    pub channels: u16,
    pub duration_sec: f64,
    pub format: String,
}

/// Information about an audio device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub id: String,
    pub name: String,
    pub is_virtual: bool,
    pub is_default: bool,
    pub is_active: bool,
}

/// Current status of a device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceStatus {
    pub device_name: String,
    pub is_active: bool,
    pub is_default: bool,
    pub error: Option<String>,
}

/// Current playback state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackState {
    pub file_id: Option<String>,
    pub is_playing: bool,
    pub is_paused: bool,
    pub current_position_sec: f64,
    pub total_duration_sec: f64,
    pub volume: f32,
    pub speed: f32,
}

impl Default for PlaybackState {
    fn default() -> Self {
        Self {
            file_id: None,
            is_playing: false,
            is_paused: false,
            current_position_sec: 0.0,
            total_duration_sec: 0.0,
            volume: 0.8,
            speed: 1.0,
        }
    }
}

/// Information about a playlist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistInfo {
    pub id: String,
    pub name: String,
    pub items: Vec<PlaylistItemInfo>,
    pub created_at: String,
}

/// Information about a playlist item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistItemInfo {
    pub id: String,
    pub audio_file_id: String,
    pub audio_file_name: String,
    pub order_index: i32,
    pub pre_silence_sec: f32,
    pub post_silence_sec: f32,
}

/// Information about a test scenario
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub turns: Vec<ScenarioTurnInfo>,
    pub created_at: String,
}

/// Information about a scenario turn
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioTurnInfo {
    pub id: String,
    pub audio_file: String,
    pub order_index: i32,
    pub delay_before_ms: f64,
    pub delay_after_ms: f64,
    pub expected_transcript: Option<String>,
    pub expected_response: Option<String>,
}

/// Data for creating/saving a scenario (without id and timestamps)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioData {
    pub name: String,
    pub description: String,
    pub turns: Vec<ScenarioTurnData>,
}

/// Data for creating/saving a scenario turn (without id)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioTurnData {
    pub audio_file: String,
    pub order_index: i32,
    pub delay_before_ms: f64,
    pub delay_after_ms: f64,
    pub expected_transcript: Option<String>,
    pub expected_response: Option<String>,
}

/// A log entry for test sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: String,
    pub session_id: String,
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

/// Serializable playlist playback status for the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistPlaybackInfo {
    pub playlist_id: String,
    pub current_item_index: usize,
    pub total_items: usize,
    pub current_file_name: String,
    pub is_playing: bool,
    pub loop_mode: bool,
}

/// Report export format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReportFormat {
    Html,
    Json,
    Csv,
}

/// Progress information for a running scenario execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioProgressInfo {
    pub session_id: String,
    pub scenario_id: String,
    pub current_turn: usize,
    pub total_turns: usize,
    pub status: String,
    pub current_file_name: Option<String>,
}
