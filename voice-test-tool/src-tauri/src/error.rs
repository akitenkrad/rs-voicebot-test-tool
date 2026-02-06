use thiserror::Error;

/// Audio-related errors
#[derive(Debug, Error)]
pub enum AudioError {
    #[error("Audio file not found: {0}")]
    FileNotFound(String),

    #[error("Unsupported audio format: {0}")]
    UnsupportedFormat(String),

    #[error("Playback failed: {0}")]
    PlaybackFailed(String),

    #[error("Decode error: {0}")]
    DecodeError(String),
}

/// Virtual device-related errors
#[derive(Debug, Error)]
pub enum DeviceError {
    #[error("Device creation failed: {0}")]
    CreationFailed(String),

    #[error("Device not found: {0}")]
    NotFound(String),

    #[error("Platform not supported: {0}")]
    PlatformUnsupported(String),
}

/// Scenario-related errors
#[derive(Debug, Error)]
pub enum ScenarioError {
    #[error("Scenario parse error: {0}")]
    ParseError(String),

    #[error("Scenario execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Invalid scenario: {0}")]
    InvalidScenario(String),
}

/// Configuration-related errors
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Config load failed: {0}")]
    LoadFailed(String),

    #[error("Config save failed: {0}")]
    SaveFailed(String),

    #[error("Config validation error: {0}")]
    ValidationError(String),
}

/// Top-level application error that wraps all domain errors
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Audio error: {0}")]
    Audio(#[from] AudioError),

    #[error("Device error: {0}")]
    Device(#[from] DeviceError),

    #[error("Scenario error: {0}")]
    Scenario(#[from] ScenarioError),

    #[error("Config error: {0}")]
    Config(#[from] ConfigError),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Convert AppError to String for Tauri command return types
impl From<AppError> for String {
    fn from(err: AppError) -> Self {
        err.to_string()
    }
}

/// Convert anyhow::Error to AppError
impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Internal(err.to_string())
    }
}
