//! Recording configuration for audio capture.

use serde::{Deserialize, Serialize};

/// Configuration for audio recording.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingConfig {
    /// Device name to record from (None = default device)
    pub device_name: Option<String>,
    /// Sample rate for recording
    pub sample_rate: u32,
    /// Number of channels
    pub channels: u16,
    /// Silence detection threshold in dB (negative value)
    pub silence_threshold_db: f32,
    /// Duration of silence required to trigger detection (ms)
    pub silence_duration_ms: u64,
    /// Maximum recording duration (seconds)
    pub max_recording_sec: u64,
}

impl Default for RecordingConfig {
    fn default() -> Self {
        Self {
            device_name: None,
            sample_rate: 16000,
            channels: 1,
            silence_threshold_db: -40.0,
            silence_duration_ms: 2000,
            max_recording_sec: 60,
        }
    }
}
