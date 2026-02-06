#![allow(unused)]

use std::path::Path;

use tauri::State;

use crate::audio::decoder;
use crate::audio::waveform;
use crate::state::AppState;
use crate::types::AudioFileInfo;

/// Load an audio file from the given path.
///
/// Decodes the file, extracts metadata, caches the decoded audio,
/// and returns the file info to the frontend.
#[tauri::command]
pub async fn load_audio_file(
    path: String,
    state: State<'_, AppState>,
) -> Result<AudioFileInfo, String> {
    let file_path = Path::new(&path);

    // Decode the audio file
    let decoded = decoder::decode_file(file_path).map_err(|e| e.to_string())?;

    let file_id = uuid::Uuid::new_v4().to_string();

    let info = AudioFileInfo {
        id: file_id.clone(),
        name: file_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string()),
        path: path.clone(),
        sample_rate: decoded.metadata.sample_rate,
        channels: decoded.metadata.channels,
        duration_sec: decoded.metadata.duration_sec,
        format: decoded.metadata.format.clone(),
    };

    tracing::info!(
        "Loaded audio file: {} ({}, {:.2}s, {}Hz, {} ch)",
        info.name,
        info.format,
        info.duration_sec,
        info.sample_rate,
        info.channels
    );

    // Cache the decoded audio for later use (playback, waveform generation)
    state.decoded_cache.lock().insert(file_id.clone(), decoded);

    // Add to the audio files list
    state.audio_files.lock().push(info.clone());

    Ok(info)
}

/// List all loaded audio files
#[tauri::command]
pub async fn list_audio_files(
    state: State<'_, AppState>,
) -> Result<Vec<AudioFileInfo>, String> {
    let files = state.audio_files.lock().clone();
    Ok(files)
}

/// Remove an audio file from the loaded list and decoded cache
#[tauri::command]
pub async fn remove_audio_file(
    file_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // Remove from audio files list
    state.audio_files.lock().retain(|f| f.id != file_id);

    // Remove from decoded cache
    state.decoded_cache.lock().remove(&file_id);

    tracing::info!("Removed audio file: {}", file_id);

    Ok(())
}

/// Get waveform data for visualization.
///
/// Generates downsampled peak data from cached decoded audio.
/// The `resolution` parameter controls how many data points are returned.
#[tauri::command]
pub async fn get_waveform_data(
    file_id: String,
    resolution: usize,
    state: State<'_, AppState>,
) -> Result<Vec<f32>, String> {
    let cache = state.decoded_cache.lock();
    let decoded = cache
        .get(&file_id)
        .ok_or_else(|| format!("Audio file not found in cache: {}", file_id))?;

    let peaks = waveform::generate_waveform_peaks(
        &decoded.samples,
        decoded.metadata.channels,
        resolution,
    );

    Ok(peaks)
}
