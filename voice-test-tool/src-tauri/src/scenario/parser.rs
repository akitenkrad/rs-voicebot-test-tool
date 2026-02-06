//! Scenario file parser
//!
//! Handles parsing and saving of test scenario files in JSON and YAML formats.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::ScenarioError;
use crate::types::{ScenarioInfo, ScenarioTurnInfo};

/// Raw scenario file structure (JSON/YAML)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioFile {
    pub name: String,
    pub description: Option<String>,
    pub metadata: Option<ScenarioMetadata>,
    pub turns: Vec<ScenarioTurnFile>,
}

/// Metadata about a scenario file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioMetadata {
    pub author: Option<String>,
    pub version: Option<String>,
    pub created_at: Option<String>,
}

/// A single turn in a scenario file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioTurnFile {
    pub audio_file: String,
    pub expected_transcript: Option<String>,
    pub expected_response: Option<String>,
    pub delay_before_ms: Option<f64>,
    pub delay_after_ms: Option<f64>,
}

/// Detect the file format from extension
fn detect_format(path: &Path) -> Result<&str, ScenarioError> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());

    match ext.as_deref() {
        Some("json") => Ok("json"),
        Some("yaml") | Some("yml") => Ok("yaml"),
        Some(other) => Err(ScenarioError::ParseError(format!(
            "Unsupported scenario file extension: {}",
            other
        ))),
        None => Err(ScenarioError::ParseError(
            "Scenario file has no extension".to_string(),
        )),
    }
}

/// Parse a scenario file (detect format from extension: .json or .yaml/.yml)
pub fn parse_scenario_file(path: &Path) -> Result<ScenarioFile, ScenarioError> {
    if !path.exists() {
        return Err(ScenarioError::ParseError(format!(
            "Scenario file not found: {}",
            path.display()
        )));
    }

    let content = std::fs::read_to_string(path).map_err(|e| {
        ScenarioError::ParseError(format!("Failed to read scenario file: {}", e))
    })?;

    let format = detect_format(path)?;

    match format {
        "json" => serde_json::from_str::<ScenarioFile>(&content).map_err(|e| {
            ScenarioError::ParseError(format!("JSON parse error: {}", e))
        }),
        "yaml" => serde_yaml::from_str::<ScenarioFile>(&content).map_err(|e| {
            ScenarioError::ParseError(format!("YAML parse error: {}", e))
        }),
        _ => unreachable!(),
    }
}

/// Save a scenario to a file (detect format from extension)
pub fn save_scenario_file(scenario: &ScenarioFile, path: &Path) -> Result<(), ScenarioError> {
    let format = detect_format(path)?;

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).map_err(|e| {
                ScenarioError::ParseError(format!(
                    "Failed to create directory {}: {}",
                    parent.display(),
                    e
                ))
            })?;
        }
    }

    let content = match format {
        "json" => serde_json::to_string_pretty(scenario).map_err(|e| {
            ScenarioError::ParseError(format!("JSON serialize error: {}", e))
        })?,
        "yaml" => serde_yaml::to_string(scenario).map_err(|e| {
            ScenarioError::ParseError(format!("YAML serialize error: {}", e))
        })?,
        _ => unreachable!(),
    };

    std::fs::write(path, content).map_err(|e| {
        ScenarioError::ParseError(format!("Failed to write scenario file: {}", e))
    })?;

    Ok(())
}

/// Convert ScenarioFile to ScenarioInfo (with IDs and timestamps)
pub fn scenario_file_to_info(file: &ScenarioFile, file_path: &Path) -> ScenarioInfo {
    let scenario_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let turns: Vec<ScenarioTurnInfo> = file
        .turns
        .iter()
        .enumerate()
        .map(|(index, turn)| {
            // Resolve audio file path relative to scenario file directory
            let audio_path = if Path::new(&turn.audio_file).is_absolute() {
                turn.audio_file.clone()
            } else if let Some(parent) = file_path.parent() {
                parent
                    .join(&turn.audio_file)
                    .to_string_lossy()
                    .to_string()
            } else {
                turn.audio_file.clone()
            };

            ScenarioTurnInfo {
                id: uuid::Uuid::new_v4().to_string(),
                audio_file: audio_path,
                order_index: index as i32,
                delay_before_ms: turn.delay_before_ms.unwrap_or(0.0),
                delay_after_ms: turn.delay_after_ms.unwrap_or(0.0),
                expected_transcript: turn.expected_transcript.clone(),
                expected_response: turn.expected_response.clone(),
            }
        })
        .collect();

    ScenarioInfo {
        id: scenario_id,
        name: file.name.clone(),
        description: file.description.clone().unwrap_or_default(),
        turns,
        created_at: file
            .metadata
            .as_ref()
            .and_then(|m| m.created_at.clone())
            .unwrap_or(now),
    }
}

/// Convert ScenarioData to ScenarioFile for saving
pub fn scenario_data_to_file(data: &crate::types::ScenarioData) -> ScenarioFile {
    ScenarioFile {
        name: data.name.clone(),
        description: Some(data.description.clone()),
        metadata: Some(ScenarioMetadata {
            author: None,
            version: Some("1.0.0".to_string()),
            created_at: Some(chrono::Utc::now().to_rfc3339()),
        }),
        turns: data
            .turns
            .iter()
            .map(|t| ScenarioTurnFile {
                audio_file: t.audio_file.clone(),
                expected_transcript: t.expected_transcript.clone(),
                expected_response: t.expected_response.clone(),
                delay_before_ms: Some(t.delay_before_ms),
                delay_after_ms: Some(t.delay_after_ms),
            })
            .collect(),
    }
}
