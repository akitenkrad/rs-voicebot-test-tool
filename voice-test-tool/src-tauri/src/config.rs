use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing;

use crate::tts::TtsConfig;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub audio: AudioConfig,
    pub virtual_device: VirtualDeviceConfig,
    pub ui: UiConfig,
    pub logging: LoggingConfig,
    pub shortcuts: ShortcutConfig,
    #[serde(default)]
    pub tts: TtsConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            audio: AudioConfig::default(),
            virtual_device: VirtualDeviceConfig::default(),
            ui: UiConfig::default(),
            logging: LoggingConfig::default(),
            shortcuts: ShortcutConfig::default(),
            tts: TtsConfig::default(),
        }
    }
}

/// Audio engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    /// Default sample rate
    pub default_sample_rate: u32,
    /// Default number of channels
    pub default_channels: u16,
    /// Buffer size in samples
    pub buffer_size: usize,
    /// Default playback speed
    pub default_playback_speed: f32,
    /// Default volume (0.0-1.0)
    pub default_volume: f32,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            default_sample_rate: 44100,
            default_channels: 1,
            buffer_size: 4096,
            default_playback_speed: 1.0,
            default_volume: 0.8,
        }
    }
}

/// Virtual device configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualDeviceConfig {
    /// Virtual device name
    pub device_name: String,
    /// Auto-create on startup
    pub auto_create_on_startup: bool,
    /// Set as default input device
    pub set_as_default: bool,
    /// Cleanup device on exit
    pub cleanup_on_exit: bool,
}

impl Default for VirtualDeviceConfig {
    fn default() -> Self {
        Self {
            device_name: "VoiceTestTool Virtual Mic".to_string(),
            auto_create_on_startup: false,
            set_as_default: false,
            cleanup_on_exit: true,
        }
    }
}

/// UI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Theme (light/dark/system)
    pub theme: String,
    /// Language
    pub language: String,
    /// Window width
    pub window_width: u32,
    /// Window height
    pub window_height: u32,
    /// Waveform display color
    pub waveform_color: String,
    /// Waveform progress color
    pub waveform_progress_color: String,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: "system".to_string(),
            language: "ja".to_string(),
            window_width: 1280,
            window_height: 800,
            waveform_color: "#4a9eff".to_string(),
            waveform_progress_color: "#1a6dcc".to_string(),
        }
    }
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level
    pub level: String,
    /// Log output directory
    pub output_dir: PathBuf,
    /// Maximum number of sessions to retain
    pub max_sessions: usize,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        let output_dir = directories::ProjectDirs::from("com", "voice-test-tool", "VoiceTestTool")
            .map(|dirs| dirs.data_dir().join("logs"))
            .unwrap_or_else(|| PathBuf::from("logs"));

        Self {
            level: "info".to_string(),
            output_dir,
            max_sessions: 100,
        }
    }
}

/// Keyboard shortcut configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutConfig {
    pub play_pause: String,
    pub stop: String,
    pub next: String,
    pub previous: String,
    pub seek_forward: String,
    pub seek_backward: String,
}

impl Default for ShortcutConfig {
    fn default() -> Self {
        Self {
            play_pause: "Space".to_string(),
            stop: "Escape".to_string(),
            next: "Ctrl+Right".to_string(),
            previous: "Ctrl+Left".to_string(),
            seek_forward: "Right".to_string(),
            seek_backward: "Left".to_string(),
        }
    }
}

/// Get the path to the config file
fn config_path() -> PathBuf {
    directories::ProjectDirs::from("com", "voice-test-tool", "VoiceTestTool")
        .map(|dirs| dirs.config_dir().join("config.toml"))
        .unwrap_or_else(|| PathBuf::from("config.toml"))
}

/// Load application config from disk, or return defaults if not found
pub fn load_config() -> AppConfig {
    let path = config_path();
    if path.exists() {
        match std::fs::read_to_string(&path) {
            Ok(content) => match toml::from_str::<AppConfig>(&content) {
                Ok(config) => return config,
                Err(e) => {
                    tracing::warn!("Failed to parse config file, using defaults: {}", e);
                }
            },
            Err(e) => {
                tracing::warn!("Failed to read config file, using defaults: {}", e);
            }
        }
    }
    AppConfig::default()
}

/// Save application config to disk
pub fn save_config(config: &AppConfig) -> Result<()> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = toml::to_string_pretty(config)?;
    std::fs::write(&path, content)?;
    Ok(())
}
