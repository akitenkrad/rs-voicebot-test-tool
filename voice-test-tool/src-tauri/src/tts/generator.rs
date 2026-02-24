use openai_tools::audio::request::{
    Audio, AudioFormat as OpenAIAudioFormat, TtsModel as OpenAITtsModel, TtsOptions, Voice,
};
use openai_tools::common::auth::{AuthProvider, OpenAIAuth};
use serde::Serialize;
use thiserror::Error;
use url::Url;

use super::config::{TtsConfig, TtsModel, TtsOutputFormat, TtsVoice};

#[derive(Debug, Error)]
pub enum TtsGeneratorError {
    #[error("API key is not configured")]
    MissingApiKey,
    #[error("TTS generation failed: {0}")]
    GenerationError(String),
}

/// Request body for Azure OpenAI TTS API.
#[derive(Debug, Serialize)]
struct AzureTtsRequest {
    model: String,
    input: String,
    voice: String,
    response_format: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    speed: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    instructions: Option<String>,
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

/// Build the correct Azure TTS URL from the user-provided base URL.
///
/// Accepts various user input formats and normalizes to:
/// `https://{resource}.openai.azure.com/openai/deployments/{deployment}/audio/speech?api-version={version}`
fn build_azure_tts_url(base_url: &str) -> Result<String, TtsGeneratorError> {
    let mut parsed = Url::parse(base_url)
        .map_err(|e| TtsGeneratorError::GenerationError(format!("Invalid base URL: {e}")))?;

    // Strip trailing `/audio/speech` or `/audio` from the path so we can re-append it cleanly
    let path = parsed.path().to_string();
    let cleaned_path = path
        .trim_end_matches('/')
        .trim_end_matches("/audio/speech")
        .trim_end_matches("/audio");
    let final_path = format!("{}/audio/speech", cleaned_path);
    parsed.set_path(&final_path);

    // Ensure `api-version` query param is present; add a default if missing
    {
        let has_api_version = parsed
            .query_pairs()
            .any(|(k, _)| k == "api-version");
        if !has_api_version {
            parsed
                .query_pairs_mut()
                .append_pair("api-version", "2024-12-01-preview");
        }
    }

    Ok(parsed.to_string())
}

/// Generate TTS audio via Azure OpenAI using a direct HTTP request.
///
/// This bypasses the `openai-tools` library's URL construction, which does not
/// correctly build the `/audio/speech` path for Azure endpoints.
async fn generate_tts_azure(
    text: &str,
    api_key: &str,
    base_url: &str,
    config: &TtsConfig,
) -> Result<Vec<u8>, TtsGeneratorError> {
    let url = build_azure_tts_url(base_url)?;

    let openai_model = to_openai_model(&config.model);
    let openai_voice = to_openai_voice(&config.default_voice);
    let openai_format = to_openai_format(&config.output_format);

    // Only include instructions when the model supports them
    let instructions = if config.model == TtsModel::Gpt4oMiniTts {
        config.default_instructions.clone()
    } else {
        None
    };

    let body = AzureTtsRequest {
        model: openai_model.as_str().to_string(),
        input: text.to_string(),
        voice: openai_voice.as_str().to_string(),
        response_format: openai_format.as_str().to_string(),
        speed: Some(config.default_speed),
        instructions,
    };

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .header("api-key", api_key)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| TtsGeneratorError::GenerationError(format!("HTTP request failed: {e}")))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_body = response
            .text()
            .await
            .unwrap_or_else(|_| "<failed to read response body>".to_string());
        return Err(TtsGeneratorError::GenerationError(format!(
            "Azure TTS API returned error (status {status}): {error_body}"
        )));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| TtsGeneratorError::GenerationError(format!("Failed to read response bytes: {e}")))?;

    Ok(bytes.to_vec())
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

    // Azure OpenAI: bypass the library and make a direct HTTP request
    if let Some(ref base_url) = config.base_url {
        if base_url.contains(".openai.azure.com") {
            return generate_tts_azure(text, api_key, base_url, config).await;
        }
    }

    // Non-Azure path: use the openai-tools library as before
    let auth = if let Some(ref base_url) = config.base_url {
        // OpenAI-compatible endpoint (e.g. Ollama, vLLM, etc.)
        AuthProvider::OpenAI(OpenAIAuth::new(api_key).with_base_url(base_url))
    } else {
        // Standard OpenAI API
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

    #[test]
    fn test_build_azure_tts_url_basic() {
        let url = build_azure_tts_url(
            "https://my-resource.openai.azure.com/openai/deployments/my-tts",
        )
        .unwrap();
        assert!(url.contains("/openai/deployments/my-tts/audio/speech"));
        assert!(url.contains("api-version="));
    }

    #[test]
    fn test_build_azure_tts_url_with_api_version() {
        let url = build_azure_tts_url(
            "https://my-resource.openai.azure.com/openai/deployments/my-tts?api-version=2024-08-01-preview",
        )
        .unwrap();
        assert!(url.contains("/openai/deployments/my-tts/audio/speech"));
        assert!(url.contains("api-version=2024-08-01-preview"));
    }

    #[test]
    fn test_build_azure_tts_url_with_audio_suffix() {
        // User already included /audio in the URL
        let url = build_azure_tts_url(
            "https://my-resource.openai.azure.com/openai/deployments/my-tts/audio",
        )
        .unwrap();
        assert!(url.contains("/openai/deployments/my-tts/audio/speech"));
        // Should NOT double the /audio path
        assert!(!url.contains("/audio/audio/"));
    }

    #[test]
    fn test_build_azure_tts_url_with_full_path() {
        // User already included /audio/speech in the URL
        let url = build_azure_tts_url(
            "https://my-resource.openai.azure.com/openai/deployments/my-tts/audio/speech?api-version=2024-08-01-preview",
        )
        .unwrap();
        assert!(url.contains("/openai/deployments/my-tts/audio/speech"));
        assert!(!url.contains("/audio/speech/audio/speech"));
        assert!(url.contains("api-version=2024-08-01-preview"));
    }
}
