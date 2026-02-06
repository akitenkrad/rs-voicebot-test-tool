//! TTS test session export functionality.
//!
//! Exports test sessions to ZIP archives containing metadata and audio files.

use std::io::Write;
use std::path::Path;

use chrono::Utc;
use serde::Serialize;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use super::types::TtsTestSession;

/// Export metadata structure for the ZIP archive.
#[derive(Debug, Serialize)]
struct ExportMetadata {
    session_id: String,
    csv_source: String,
    created_at: String,
    tts_config: TtsConfigExport,
    recording_config: RecordingConfigExport,
    test_cases: Vec<TestCaseExport>,
}

/// TTS configuration for export.
#[derive(Debug, Serialize)]
struct TtsConfigExport {
    model: String,
    voice: String,
    speed: f32,
    instructions: Option<String>,
}

/// Recording configuration for export.
#[derive(Debug, Serialize)]
struct RecordingConfigExport {
    device: Option<String>,
    sample_rate: u32,
    silence_threshold_db: f32,
    silence_duration_ms: u64,
}

/// Test case result for export.
#[derive(Debug, Serialize)]
struct TestCaseExport {
    id: String,
    input_text: String,
    tts_audio_file: Option<String>,
    response_audio_file: Option<String>,
    tts_duration_sec: f64,
    response_duration_sec: Option<f64>,
    timestamp: String,
    error: Option<String>,
}

/// Export a TTS test session to a ZIP archive.
///
/// Creates a ZIP file with the following structure:
/// ```text
/// export_YYYYMMDD_HHMMSS.zip
/// ├── metadata.json           # Session info + all test results
/// ├── tts/
/// │   ├── 001_input.wav       # Generated TTS audio (if available)
/// │   ├── 002_input.wav
/// │   └── ...
/// └── responses/
///     ├── 001_response.wav    # Recorded response audio (if available)
///     ├── 002_response.wav
///     └── ...
/// ```
///
/// # Arguments
/// * `session` - The TTS test session to export
/// * `output_path` - Path where the ZIP file should be created
///
/// # Returns
/// * `Ok(String)` - The path to the created ZIP file
/// * `Err(String)` - Error message if export failed
pub fn export_session_to_zip(session: &TtsTestSession, output_path: &Path) -> Result<String, String> {
    // Create ZIP file
    let file =
        std::fs::File::create(output_path).map_err(|e| format!("Failed to create file: {}", e))?;
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    // Create metadata
    let mut test_cases_export = Vec::new();

    for (index, result) in session.results.iter().enumerate() {
        let id = format!("{:03}", index + 1);

        // Determine file paths based on whether audio data is available
        let tts_file = if !result.tts_audio.is_empty() {
            Some(format!("tts/{}_input.wav", id))
        } else {
            None
        };

        let response_file = if result.response_audio.is_some() {
            Some(format!("responses/{}_response.wav", id))
        } else {
            None
        };

        // Add TTS audio to ZIP if available
        if let Some(ref tts_path) = tts_file {
            if !result.tts_audio.is_empty() {
                zip.start_file(tts_path, options)
                    .map_err(|e| format!("Failed to add TTS file: {}", e))?;
                zip.write_all(&result.tts_audio)
                    .map_err(|e| format!("Failed to write TTS data: {}", e))?;
            }
        }

        // Add response audio to ZIP if available
        if let Some(ref response_path) = response_file {
            if let Some(ref response_data) = result.response_audio {
                zip.start_file(response_path, options)
                    .map_err(|e| format!("Failed to add response file: {}", e))?;
                zip.write_all(response_data)
                    .map_err(|e| format!("Failed to write response data: {}", e))?;
            }
        }

        test_cases_export.push(TestCaseExport {
            id,
            input_text: result.input_text.clone(),
            tts_audio_file: tts_file,
            response_audio_file: response_file,
            tts_duration_sec: result.tts_duration_sec,
            response_duration_sec: result.response_duration_sec,
            timestamp: result.started_at.to_rfc3339(),
            error: result.error.clone(),
        });
    }

    // Create metadata JSON
    let metadata = ExportMetadata {
        session_id: session.id.clone(),
        csv_source: session.csv_path.to_string_lossy().to_string(),
        created_at: Utc::now().to_rfc3339(),
        tts_config: TtsConfigExport {
            model: format!("{:?}", session.config.model),
            voice: format!("{:?}", session.config.default_voice),
            speed: session.config.default_speed,
            instructions: session.config.default_instructions.clone(),
        },
        recording_config: RecordingConfigExport {
            device: session.recording_config.device_name.clone(),
            sample_rate: session.recording_config.sample_rate,
            silence_threshold_db: session.recording_config.silence_threshold_db,
            silence_duration_ms: session.recording_config.silence_duration_ms,
        },
        test_cases: test_cases_export,
    };

    // Add metadata.json to ZIP
    let metadata_json = serde_json::to_string_pretty(&metadata)
        .map_err(|e| format!("Failed to serialize metadata: {}", e))?;
    zip.start_file("metadata.json", options)
        .map_err(|e| format!("Failed to add metadata file: {}", e))?;
    zip.write_all(metadata_json.as_bytes())
        .map_err(|e| format!("Failed to write metadata: {}", e))?;

    // Finalize ZIP
    zip.finish()
        .map_err(|e| format!("Failed to finalize ZIP: {}", e))?;

    tracing::info!(
        "Exported TTS test session {} to {}",
        session.id,
        output_path.display()
    );

    Ok(output_path.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::RecordingConfig;
    use crate::tts::{TtsConfig, TtsTestResult, TtsTestStatus};
    use chrono::Utc;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn create_test_session() -> TtsTestSession {
        TtsTestSession {
            id: "test-session-123".to_string(),
            csv_path: PathBuf::from("/test/path.csv"),
            test_cases: vec![],
            results: vec![TtsTestResult {
                test_case_id: "001".to_string(),
                input_text: "Hello world".to_string(),
                tts_duration_sec: 1.5,
                response_duration_sec: Some(2.0),
                started_at: Utc::now(),
                ended_at: Utc::now(),
                error: None,
                tts_audio: vec![0u8; 100], // Dummy audio data
                response_audio: Some(vec![0u8; 100]),
            }],
            config: TtsConfig::default(),
            recording_config: RecordingConfig::default(),
            status: TtsTestStatus::Completed,
            started_at: Utc::now(),
            ended_at: Some(Utc::now()),
        }
    }

    #[test]
    fn test_export_session_to_zip() {
        let session = create_test_session();
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("test_export.zip");

        let result = export_session_to_zip(&session, &output_path);
        assert!(result.is_ok());
        assert!(output_path.exists());
    }

    #[test]
    fn test_export_empty_session() {
        let mut session = create_test_session();
        session.results = vec![];

        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("empty_export.zip");

        let result = export_session_to_zip(&session, &output_path);
        assert!(result.is_ok());
        assert!(output_path.exists());
    }
}
