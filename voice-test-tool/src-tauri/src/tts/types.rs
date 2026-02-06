//! TTS test execution types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::config::TtsConfig;
use super::csv_parser::TtsTestCase;
use crate::audio::RecordingConfig;

/// A TTS test session containing all test cases and results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsTestSession {
    /// Unique identifier for this session
    pub id: String,
    /// Path to the source CSV file
    pub csv_path: PathBuf,
    /// List of test cases to execute
    pub test_cases: Vec<TtsTestCase>,
    /// Results for completed test cases
    pub results: Vec<TtsTestResult>,
    /// TTS configuration used for this session
    pub config: TtsConfig,
    /// Recording configuration used for this session
    pub recording_config: RecordingConfig,
    /// Current status of the session
    pub status: TtsTestStatus,
    /// When the session was started
    pub started_at: DateTime<Utc>,
    /// When the session ended (if completed/aborted/failed)
    pub ended_at: Option<DateTime<Utc>>,
}

/// Result of a single TTS test case execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsTestResult {
    /// ID of the test case this result belongs to
    pub test_case_id: String,
    /// The input text that was converted to speech
    pub input_text: String,
    /// Duration of the generated TTS audio in seconds
    pub tts_duration_sec: f64,
    /// Duration of the recorded response in seconds (if any)
    pub response_duration_sec: Option<f64>,
    /// When this test case started
    pub started_at: DateTime<Utc>,
    /// When this test case ended
    pub ended_at: DateTime<Utc>,
    /// Error message if this test case failed
    pub error: Option<String>,
    /// Generated TTS audio data (WAV format)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tts_audio: Vec<u8>,
    /// Recorded response audio data (WAV format, if any)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_audio: Option<Vec<u8>>,
}

/// Status of a TTS test session.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum TtsTestStatus {
    /// Session is waiting to start
    Pending,
    /// Session is currently running
    Running {
        /// Index of the currently executing test case
        current_index: usize,
    },
    /// Session is paused
    Paused {
        /// Index of the test case that was running when paused
        current_index: usize,
    },
    /// Session completed successfully
    Completed,
    /// Session was manually aborted
    Aborted,
    /// Session failed with an error
    Failed {
        /// Error message describing the failure
        message: String,
    },
}

impl Default for TtsTestStatus {
    fn default() -> Self {
        Self::Pending
    }
}

impl TtsTestStatus {
    /// Convert status to a string representation for logging.
    pub fn as_str(&self) -> &str {
        match self {
            TtsTestStatus::Pending => "pending",
            TtsTestStatus::Running { .. } => "running",
            TtsTestStatus::Paused { .. } => "paused",
            TtsTestStatus::Completed => "completed",
            TtsTestStatus::Aborted => "aborted",
            TtsTestStatus::Failed { .. } => "failed",
        }
    }
}
