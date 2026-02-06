//! Test logging and report generation
//!
//! Provides test session management, event logging, and report export
//! in JSON, CSV, and HTML formats.

use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::ScenarioError;
use crate::types::LogEntry;

/// Data for a test session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSession {
    pub id: String,
    pub scenario_id: String,
    pub scenario_name: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub status: String,
}

/// Internal storage for a test session and its events
pub struct TestSessionData {
    pub session: TestSession,
    pub events: Vec<LogEntry>,
}

/// Test logger that manages sessions and their events
pub struct TestLogger {
    sessions: HashMap<String, TestSessionData>,
}

impl TestLogger {
    /// Create a new TestLogger
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    /// Create a new test session and return the session_id
    pub fn create_session(&mut self, scenario_id: &str, scenario_name: &str) -> String {
        let session_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        let session = TestSession {
            id: session_id.clone(),
            scenario_id: scenario_id.to_string(),
            scenario_name: scenario_name.to_string(),
            started_at: now,
            ended_at: None,
            status: "running".to_string(),
        };

        let data = TestSessionData {
            session,
            events: Vec::new(),
        };

        self.sessions.insert(session_id.clone(), data);

        tracing::info!(
            "Created test session: {} for scenario: {}",
            session_id,
            scenario_name
        );

        session_id
    }

    /// Log an event for a session
    pub fn log_event(
        &mut self,
        session_id: &str,
        level: &str,
        message: &str,
        details: Option<serde_json::Value>,
    ) {
        if let Some(data) = self.sessions.get_mut(session_id) {
            let entry = LogEntry {
                id: uuid::Uuid::new_v4().to_string(),
                session_id: session_id.to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
                level: level.to_string(),
                message: message.to_string(),
                details,
            };

            data.events.push(entry);
        } else {
            tracing::warn!(
                "Attempted to log event for unknown session: {}",
                session_id
            );
        }
    }

    /// Mark a session as ended with a final status
    pub fn end_session(&mut self, session_id: &str, status: &str) {
        if let Some(data) = self.sessions.get_mut(session_id) {
            data.session.ended_at = Some(chrono::Utc::now().to_rfc3339());
            data.session.status = status.to_string();
            tracing::info!("Ended test session: {} with status: {}", session_id, status);
        }
    }

    /// Get logs for a specific session
    pub fn get_session_logs(&self, session_id: &str) -> Vec<LogEntry> {
        self.sessions
            .get(session_id)
            .map(|data| data.events.clone())
            .unwrap_or_default()
    }

    /// Get a specific session
    pub fn get_session(&self, session_id: &str) -> Option<TestSession> {
        self.sessions
            .get(session_id)
            .map(|data| data.session.clone())
    }

    /// List all test sessions
    pub fn list_sessions(&self) -> Vec<TestSession> {
        self.sessions
            .values()
            .map(|data| data.session.clone())
            .collect()
    }

    /// Export session data as JSON
    pub fn export_json(&self, session_id: &str, path: &Path) -> Result<(), ScenarioError> {
        let data = self.sessions.get(session_id).ok_or_else(|| {
            ScenarioError::ExecutionFailed(format!("Session not found: {}", session_id))
        })?;

        #[derive(Serialize)]
        struct ExportData {
            session: TestSession,
            events: Vec<LogEntry>,
        }

        let export = ExportData {
            session: data.session.clone(),
            events: data.events.clone(),
        };

        let content = serde_json::to_string_pretty(&export).map_err(|e| {
            ScenarioError::ExecutionFailed(format!("JSON serialize error: {}", e))
        })?;

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    ScenarioError::ExecutionFailed(format!(
                        "Failed to create directory: {}",
                        e
                    ))
                })?;
            }
        }

        std::fs::write(path, content).map_err(|e| {
            ScenarioError::ExecutionFailed(format!("Failed to write JSON report: {}", e))
        })?;

        tracing::info!("Exported JSON report to: {}", path.display());
        Ok(())
    }

    /// Export session data as CSV
    pub fn export_csv(&self, session_id: &str, path: &Path) -> Result<(), ScenarioError> {
        let data = self.sessions.get(session_id).ok_or_else(|| {
            ScenarioError::ExecutionFailed(format!("Session not found: {}", session_id))
        })?;

        let mut csv_content = String::new();

        // Header
        csv_content.push_str("id,session_id,timestamp,level,message,details\n");

        // Rows
        for event in &data.events {
            let details_str = event
                .details
                .as_ref()
                .map(|d| d.to_string())
                .unwrap_or_default()
                // Escape double quotes in CSV
                .replace('"', "\"\"");

            csv_content.push_str(&format!(
                "{},{},{},{},\"{}\",\"{}\"\n",
                event.id,
                event.session_id,
                event.timestamp,
                event.level,
                event.message.replace('"', "\"\""),
                details_str,
            ));
        }

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    ScenarioError::ExecutionFailed(format!(
                        "Failed to create directory: {}",
                        e
                    ))
                })?;
            }
        }

        std::fs::write(path, csv_content).map_err(|e| {
            ScenarioError::ExecutionFailed(format!("Failed to write CSV report: {}", e))
        })?;

        tracing::info!("Exported CSV report to: {}", path.display());
        Ok(())
    }

    /// Export session data as a simple HTML table
    pub fn export_html(&self, session_id: &str, path: &Path) -> Result<(), ScenarioError> {
        let data = self.sessions.get(session_id).ok_or_else(|| {
            ScenarioError::ExecutionFailed(format!("Session not found: {}", session_id))
        })?;

        let session = &data.session;

        let mut html = String::new();
        html.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n");
        html.push_str("  <meta charset=\"UTF-8\">\n");
        html.push_str("  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
        html.push_str(&format!(
            "  <title>Test Report - {}</title>\n",
            escape_html(&session.scenario_name)
        ));
        html.push_str("  <style>\n");
        html.push_str("    body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 20px; background: #f5f5f5; }\n");
        html.push_str("    h1 { color: #333; }\n");
        html.push_str("    .summary { background: white; padding: 16px; border-radius: 8px; margin-bottom: 20px; box-shadow: 0 1px 3px rgba(0,0,0,0.1); }\n");
        html.push_str("    .summary dt { font-weight: bold; color: #666; }\n");
        html.push_str("    .summary dd { margin-bottom: 8px; }\n");
        html.push_str("    table { width: 100%; border-collapse: collapse; background: white; border-radius: 8px; overflow: hidden; box-shadow: 0 1px 3px rgba(0,0,0,0.1); }\n");
        html.push_str("    th { background: #4a5568; color: white; padding: 12px 16px; text-align: left; }\n");
        html.push_str("    td { padding: 10px 16px; border-bottom: 1px solid #e2e8f0; }\n");
        html.push_str("    tr:hover { background: #f7fafc; }\n");
        html.push_str("    .level-info { color: #2b6cb0; }\n");
        html.push_str("    .level-warn { color: #c05621; }\n");
        html.push_str("    .level-error { color: #c53030; }\n");
        html.push_str("    .status-completed { color: #2f855a; font-weight: bold; }\n");
        html.push_str("    .status-failed { color: #c53030; font-weight: bold; }\n");
        html.push_str("    .status-aborted { color: #c05621; font-weight: bold; }\n");
        html.push_str("    .status-running { color: #2b6cb0; font-weight: bold; }\n");
        html.push_str("  </style>\n");
        html.push_str("</head>\n<body>\n");

        // Title
        html.push_str(&format!(
            "  <h1>Test Report: {}</h1>\n",
            escape_html(&session.scenario_name)
        ));

        // Session summary
        html.push_str("  <div class=\"summary\">\n");
        html.push_str("    <dl>\n");
        html.push_str(&format!(
            "      <dt>Session ID</dt><dd>{}</dd>\n",
            escape_html(&session.id)
        ));
        html.push_str(&format!(
            "      <dt>Scenario</dt><dd>{}</dd>\n",
            escape_html(&session.scenario_name)
        ));
        html.push_str(&format!(
            "      <dt>Started</dt><dd>{}</dd>\n",
            escape_html(&session.started_at)
        ));
        html.push_str(&format!(
            "      <dt>Ended</dt><dd>{}</dd>\n",
            session
                .ended_at
                .as_deref()
                .map(escape_html)
                .unwrap_or_else(|| "-".to_string())
        ));
        html.push_str(&format!(
            "      <dt>Status</dt><dd class=\"status-{}\">{}</dd>\n",
            escape_html(&session.status),
            escape_html(&session.status)
        ));
        html.push_str(&format!(
            "      <dt>Total Events</dt><dd>{}</dd>\n",
            data.events.len()
        ));
        html.push_str("    </dl>\n");
        html.push_str("  </div>\n");

        // Events table
        html.push_str("  <h2>Events</h2>\n");
        html.push_str("  <table>\n");
        html.push_str("    <thead>\n");
        html.push_str("      <tr><th>Timestamp</th><th>Level</th><th>Message</th><th>Details</th></tr>\n");
        html.push_str("    </thead>\n");
        html.push_str("    <tbody>\n");

        for event in &data.events {
            let level_class = format!("level-{}", event.level);
            let details_str = event
                .details
                .as_ref()
                .map(|d| serde_json::to_string_pretty(d).unwrap_or_default())
                .unwrap_or_default();

            html.push_str(&format!(
                "      <tr><td>{}</td><td class=\"{}\">{}</td><td>{}</td><td><pre>{}</pre></td></tr>\n",
                escape_html(&event.timestamp),
                level_class,
                escape_html(&event.level),
                escape_html(&event.message),
                escape_html(&details_str),
            ));
        }

        html.push_str("    </tbody>\n");
        html.push_str("  </table>\n");
        html.push_str("</body>\n</html>\n");

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    ScenarioError::ExecutionFailed(format!(
                        "Failed to create directory: {}",
                        e
                    ))
                })?;
            }
        }

        std::fs::write(path, html).map_err(|e| {
            ScenarioError::ExecutionFailed(format!("Failed to write HTML report: {}", e))
        })?;

        tracing::info!("Exported HTML report to: {}", path.display());
        Ok(())
    }
}

/// Escape HTML special characters
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
