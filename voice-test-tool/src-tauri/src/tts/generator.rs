use openai_tools::audio::request::{
    Audio, AudioFormat as OpenAIAudioFormat, TtsModel as OpenAITtsModel, TtsOptions, Voice,
};
use openai_tools::common::auth::{AuthProvider, OpenAIAuth};
use thiserror::Error;

use super::config::{TtsConfig, TtsModel, TtsOutputFormat, TtsVoice};

#[derive(Debug, Error)]
pub enum TtsGeneratorError {
    #[error("API key is not configured")]
    MissingApiKey,
    #[error("TTS generation failed: {0}")]
    GenerationError(String),
}

/// Convert our TtsModel enum to openai-tools TtsModel
fn to_openai_model(model: &TtsModel) -> OpenAITtsModel {
    match model {
        TtsModel::Tts1 => OpenAITtsModel::Tts1,
        TtsModel::Tts1Hd => OpenAITtsModel::Tts1Hd,
        TtsModel::Gpt4oMiniTts => OpenAITtsModel::Gpt4oMiniTts,
    }
}

/// Convert our TtsVoice enum to openai-tools Voice
fn to_openai_voice(voice: &TtsVoice) -> Voice {
    match voice {
        TtsVoice::Alloy => Voice::Alloy,
        TtsVoice::Ash => Voice::Ash,
        TtsVoice::Ballad => Voice::Ballad,
        TtsVoice::Coral => Voice::Coral,
        TtsVoice::Echo => Voice::Echo,
        TtsVoice::Fable => Voice::Fable,
        TtsVoice::Nova => Voice::Nova,
        TtsVoice::Onyx => Voice::Onyx,
        TtsVoice::Sage => Voice::Sage,
        TtsVoice::Shimmer => Voice::Shimmer,
        TtsVoice::Verse => Voice::Verse,
    }
}

/// Convert our TtsOutputFormat enum to openai-tools AudioFormat
fn to_openai_format(format: &TtsOutputFormat) -> OpenAIAudioFormat {
    match format {
        TtsOutputFormat::Mp3 => OpenAIAudioFormat::Mp3,
        TtsOutputFormat::Opus => OpenAIAudioFormat::Opus,
        TtsOutputFormat::Aac => OpenAIAudioFormat::Aac,
        TtsOutputFormat::Flac => OpenAIAudioFormat::Flac,
        TtsOutputFormat::Wav => OpenAIAudioFormat::Wav,
        TtsOutputFormat::Pcm => OpenAIAudioFormat::Pcm,
    }
}

/// Generate TTS audio from text using the OpenAI API.
///
/// # Arguments
///
/// * `text` - The text to convert to speech
/// * `config` - TTS configuration containing API key and settings
///
/// # Returns
///
/// Audio bytes as `Vec<u8>`, or an error if generation fails.
pub async fn generate_tts(text: &str, config: &TtsConfig) -> Result<Vec<u8>, TtsGeneratorError> {
    let api_key = config
        .api_key
        .as_ref()
        .ok_or(TtsGeneratorError::MissingApiKey)?;

    // Create the auth provider — use Azure/custom endpoint if base_url is set
    let auth = if let Some(ref base_url) = config.base_url {
        AuthProvider::from_url_with_key(base_url, api_key)
    } else {
        AuthProvider::OpenAI(OpenAIAuth::new(api_key))
    };
    let audio = Audio::with_auth(auth);

    // Build TTS options from config
    let options = TtsOptions {
        model: to_openai_model(&config.model),
        voice: to_openai_voice(&config.default_voice),
        response_format: to_openai_format(&config.output_format),
        speed: Some(config.default_speed),
        instructions: config.default_instructions.clone(),
    };

    // Call the TTS API
    let bytes = audio
        .text_to_speech(text, options)
        .await
        .map_err(|e| TtsGeneratorError::GenerationError(e.to_string()))?;

    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_conversion() {
        assert!(matches!(
            to_openai_model(&TtsModel::Tts1),
            OpenAITtsModel::Tts1
        ));
        assert!(matches!(
            to_openai_model(&TtsModel::Tts1Hd),
            OpenAITtsModel::Tts1Hd
        ));
        assert!(matches!(
            to_openai_model(&TtsModel::Gpt4oMiniTts),
            OpenAITtsModel::Gpt4oMiniTts
        ));
    }

    #[test]
    fn test_voice_conversion() {
        assert!(matches!(to_openai_voice(&TtsVoice::Alloy), Voice::Alloy));
        assert!(matches!(to_openai_voice(&TtsVoice::Nova), Voice::Nova));
        assert!(matches!(
            to_openai_voice(&TtsVoice::Shimmer),
            Voice::Shimmer
        ));
    }

    #[test]
    fn test_format_conversion() {
        assert!(matches!(
            to_openai_format(&TtsOutputFormat::Mp3),
            OpenAIAudioFormat::Mp3
        ));
        assert!(matches!(
            to_openai_format(&TtsOutputFormat::Wav),
            OpenAIAudioFormat::Wav
        ));
        assert!(matches!(
            to_openai_format(&TtsOutputFormat::Flac),
            OpenAIAudioFormat::Flac
        ));
    }
}
