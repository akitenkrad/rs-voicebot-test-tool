#![allow(unused)]

use tauri::State;

use crate::audio::player::PlayerCommand;
use crate::state::AppState;
use crate::types::{AudioSystemStatus, PlaybackState};

/// Build an error message for when the audio player is not available,
/// including the original initialization error if known.
fn player_unavailable_error(state: &AppState) -> String {
    let init_err = state.audio_init_error.lock();
    match init_err.as_deref() {
        Some(reason) => format!("Audio player not available: {}", reason),
        None => "Audio player not available".to_string(),
    }
}

/// Start playback of the specified audio file.
///
/// If the audio is not already loaded into the player, it will be loaded
/// from the decoded cache first.
#[tauri::command]
pub async fn play(
    file_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let player_lock = state.player.lock();
    let player = player_lock
        .as_ref()
        .ok_or_else(|| player_unavailable_error(&state))?;

    // Check if we need to load the audio into the player
    let current_file_id = state.playback_state.lock().file_id.clone();
    let needs_load = current_file_id.as_deref() != Some(&file_id);

    if needs_load {
        // Get decoded audio from cache
        let decoded = {
            let cache = state.decoded_cache.lock();
            cache
                .get(&file_id)
                .cloned()
                .ok_or_else(|| format!("Audio file not in cache: {}", file_id))?
        };

        let duration = decoded.metadata.duration_sec;

        // Load into the player
        player.load(decoded).map_err(|e| e.to_string())?;

        // Update playback state
        let mut playback = state.playback_state.lock();
        playback.file_id = Some(file_id.clone());
        playback.total_duration_sec = duration;
        playback.current_position_sec = 0.0;
    }

    // Send play command
    player.send_command(PlayerCommand::Play);

    // Update state
    let mut playback = state.playback_state.lock();
    playback.is_playing = true;
    playback.is_paused = false;

    tracing::info!("Play command sent for file: {}", file_id);
    Ok(())
}

/// Pause the current playback
#[tauri::command]
pub async fn pause(
    state: State<'_, AppState>,
) -> Result<(), String> {
    let player_lock = state.player.lock();
    let player = player_lock
        .as_ref()
        .ok_or_else(|| player_unavailable_error(&state))?;

    player.send_command(PlayerCommand::Pause);

    let mut playback = state.playback_state.lock();
    playback.is_playing = false;
    playback.is_paused = true;

    tracing::info!("Pause command sent");
    Ok(())
}

/// Stop the current playback
#[tauri::command]
pub async fn stop(
    state: State<'_, AppState>,
) -> Result<(), String> {
    let player_lock = state.player.lock();
    let player = player_lock
        .as_ref()
        .ok_or_else(|| player_unavailable_error(&state))?;

    player.send_command(PlayerCommand::Stop);

    let mut playback = state.playback_state.lock();
    playback.is_playing = false;
    playback.is_paused = false;
    playback.current_position_sec = 0.0;

    tracing::info!("Stop command sent");
    Ok(())
}

/// Seek to a specific position in the current playback
#[tauri::command]
pub async fn seek(
    position_sec: f64,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let player_lock = state.player.lock();
    let player = player_lock
        .as_ref()
        .ok_or_else(|| player_unavailable_error(&state))?;

    player.send_command(PlayerCommand::Seek(position_sec));

    let mut playback = state.playback_state.lock();
    playback.current_position_sec = position_sec;

    tracing::info!("Seek command sent: {:.2}s", position_sec);
    Ok(())
}

/// Set the playback speed (0.5 - 2.0)
#[tauri::command]
pub async fn set_playback_speed(
    speed: f32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let clamped = speed.clamp(0.5, 2.0);

    let player_lock = state.player.lock();
    let player = player_lock
        .as_ref()
        .ok_or_else(|| player_unavailable_error(&state))?;

    player.send_command(PlayerCommand::SetSpeed(clamped));

    state.playback_state.lock().speed = clamped;

    tracing::info!("Playback speed set to {:.2}x", clamped);
    Ok(())
}

/// Set the playback volume (0.0 - 1.0)
#[tauri::command]
pub async fn set_volume(
    volume: f32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let clamped = volume.clamp(0.0, 1.0);

    let player_lock = state.player.lock();
    let player = player_lock
        .as_ref()
        .ok_or_else(|| player_unavailable_error(&state))?;

    player.send_command(PlayerCommand::SetVolume(clamped));

    state.playback_state.lock().volume = clamped;

    tracing::info!("Volume set to {:.2}", clamped);
    Ok(())
}

/// Get the current playback state.
///
/// This synchronizes with the player's real-time status to return
/// an accurate current position.
#[tauri::command]
pub async fn get_playback_state(
    state: State<'_, AppState>,
) -> Result<PlaybackState, String> {
    // Get the real-time status from the player if available
    let player_lock = state.player.lock();
    if let Some(ref player) = *player_lock {
        let status = player.get_status();
        let mut playback = state.playback_state.lock();

        // Sync position and playing state from the player thread
        playback.current_position_sec = status.position_sec;
        playback.is_playing = status.is_playing;
        playback.is_paused = status.is_paused;

        Ok(playback.clone())
    } else {
        // No player available, return current state as-is
        Ok(state.playback_state.lock().clone())
    }
}

/// Get the audio system status (whether the player initialized successfully).
#[tauri::command]
pub async fn get_audio_system_status(
    state: State<'_, AppState>,
) -> Result<AudioSystemStatus, String> {
    let player_available = state.player.lock().is_some();
    let error = state.audio_init_error.lock().clone();
    Ok(AudioSystemStatus {
        available: player_available,
        error,
    })
}
