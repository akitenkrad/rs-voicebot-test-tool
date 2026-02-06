use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TtsModel {
    Tts1,
    Tts1Hd,
    Gpt4oMiniTts,
}

impl Default for TtsModel {
    fn default() -> Self {
        TtsModel::Gpt4oMiniTts
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TtsVoice {
    Alloy,
    Ash,
    Ballad,
    Coral,
    Echo,
    Fable,
    Nova,
    Onyx,
    Sage,
    Shimmer,
    Verse,
}

impl Default for TtsVoice {
    fn default() -> Self {
        TtsVoice::Alloy
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TtsOutputFormat {
    Mp3,
    Opus,
    Aac,
    Flac,
    Wav,
    Pcm,
}

impl Default for TtsOutputFormat {
    fn default() -> Self {
        TtsOutputFormat::Wav
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsConfig {
    pub api_key: Option<String>,
    pub model: TtsModel,
    pub default_voice: TtsVoice,
    pub default_speed: f32,
    pub default_instructions: Option<String>,
    pub output_format: TtsOutputFormat,
}

impl Default for TtsConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            model: TtsModel::default(),
            default_voice: TtsVoice::default(),
            default_speed: 1.0,
            default_instructions: None,
            output_format: TtsOutputFormat::default(),
        }
    }
}
