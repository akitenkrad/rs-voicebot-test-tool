use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tauri::State;

use crate::audio::player::PlayerCommand;
use crate::state::{AppState, PlaylistPlaybackState};
use crate::types::{PlaylistInfo, PlaylistPlaybackInfo};

/// Create a new playlist
#[tauri::command]
pub async fn create_playlist(
    name: String,
    state: State<'_, AppState>,
) -> Result<PlaylistInfo, String> {
    let playlist = PlaylistInfo {
        id: uuid::Uuid::new_v4().to_string(),
        name,
        items: Vec::new(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    state.playlists.lock().push(playlist.clone());
    tracing::info!("Created playlist: {}", playlist.id);
    Ok(playlist)
}

/// Add an audio file to a playlist
#[tauri::command]
pub async fn add_to_playlist(
    playlist_id: String,
    file_id: String,
    pre_silence_sec: f32,
    post_silence_sec: f32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut playlists = state.playlists.lock();
    let playlist = playlists
        .iter_mut()
        .find(|p| p.id == playlist_id)
        .ok_or_else(|| format!("Playlist not found: {}", playlist_id))?;

    let order_index = playlist.items.len() as i32;
    let audio_files = state.audio_files.lock();
    let audio_file_name = audio_files
        .iter()
        .find(|f| f.id == file_id)
        .map(|f| f.name.clone())
        .unwrap_or_else(|| "unknown".to_string());

    playlist.items.push(crate::types::PlaylistItemInfo {
        id: uuid::Uuid::new_v4().to_string(),
        audio_file_id: file_id,
        audio_file_name,
        order_index,
        pre_silence_sec,
        post_silence_sec,
    });

    tracing::info!("Added item to playlist {}", playlist_id);
    Ok(())
}

/// List all playlists
#[tauri::command]
pub async fn list_playlists(state: State<'_, AppState>) -> Result<Vec<PlaylistInfo>, String> {
    let playlists = state.playlists.lock();
    Ok(playlists.clone())
}

/// Get a specific playlist by ID
#[tauri::command]
pub async fn get_playlist(
    playlist_id: String,
    state: State<'_, AppState>,
) -> Result<PlaylistInfo, String> {
    let playlists = state.playlists.lock();
    playlists
        .iter()
        .find(|p| p.id == playlist_id)
        .cloned()
        .ok_or_else(|| format!("Playlist not found: {}", playlist_id))
}

/// Delete a playlist
#[tauri::command]
pub async fn delete_playlist(
    playlist_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // If this playlist is currently playing, stop it first
    {
        let playback = state.playlist_playback.lock();
        if let Some(ref pb) = *playback {
            if pb.playlist_id == playlist_id {
                pb.cancel.store(true, Ordering::SeqCst);
            }
        }
    }

    let mut playlists = state.playlists.lock();
    let original_len = playlists.len();
    playlists.retain(|p| p.id != playlist_id);

    if playlists.len() == original_len {
        return Err(format!("Playlist not found: {}", playlist_id));
    }

    tracing::info!("Deleted playlist: {}", playlist_id);
    Ok(())
}

/// Remove an item from a playlist
#[tauri::command]
pub async fn remove_from_playlist(
    playlist_id: String,
    item_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut playlists = state.playlists.lock();
    let playlist = playlists
        .iter_mut()
        .find(|p| p.id == playlist_id)
        .ok_or_else(|| format!("Playlist not found: {}", playlist_id))?;

    let original_len = playlist.items.len();
    playlist.items.retain(|item| item.id != item_id);

    if playlist.items.len() == original_len {
        return Err(format!("Playlist item not found: {}", item_id));
    }

    // Re-index remaining items
    for (i, item) in playlist.items.iter_mut().enumerate() {
        item.order_index = i as i32;
    }

    tracing::info!(
        "Removed item {} from playlist {}",
        item_id,
        playlist_id
    );
    Ok(())
}

/// Reorder playlist items
#[tauri::command]
pub async fn reorder_playlist(
    playlist_id: String,
    item_ids: Vec<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut playlists = state.playlists.lock();
    let playlist = playlists
        .iter_mut()
        .find(|p| p.id == playlist_id)
        .ok_or_else(|| format!("Playlist not found: {}", playlist_id))?;

    // Validate that all provided item_ids exist in the playlist
    let existing_ids: Vec<String> = playlist.items.iter().map(|i| i.id.clone()).collect();
    for id in &item_ids {
        if !existing_ids.contains(id) {
            return Err(format!("Item not found in playlist: {}", id));
        }
    }

    // Reorder items to match the provided order
    let mut reordered = Vec::with_capacity(playlist.items.len());
    for (new_index, id) in item_ids.iter().enumerate() {
        if let Some(mut item) = playlist.items.iter().find(|i| i.id == *id).cloned() {
            item.order_index = new_index as i32;
            reordered.push(item);
        }
    }

    // Append any items not mentioned in item_ids at the end (preserving their relative order)
    let mut next_index = reordered.len() as i32;
    for item in &playlist.items {
        if !item_ids.contains(&item.id) {
            let mut remaining_item = item.clone();
            remaining_item.order_index = next_index;
            next_index += 1;
            reordered.push(remaining_item);
        }
    }

    playlist.items = reordered;

    tracing::info!("Reordered playlist {}", playlist_id);
    Ok(())
}

/// Update silence settings for a playlist item
#[tauri::command]
pub async fn update_playlist_item_silence(
    playlist_id: String,
    item_id: String,
    pre_silence_sec: f32,
    post_silence_sec: f32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut playlists = state.playlists.lock();
    let playlist = playlists
        .iter_mut()
        .find(|p| p.id == playlist_id)
        .ok_or_else(|| format!("Playlist not found: {}", playlist_id))?;

    let item = playlist
        .items
        .iter_mut()
        .find(|i| i.id == item_id)
        .ok_or_else(|| format!("Playlist item not found: {}", item_id))?;

    item.pre_silence_sec = pre_silence_sec;
    item.post_silence_sec = post_silence_sec;

    tracing::info!(
        "Updated silence for item {} in playlist {}: pre={:.2}s, post={:.2}s",
        item_id,
        playlist_id,
        pre_silence_sec,
        post_silence_sec
    );
    Ok(())
}

/// Play a playlist
#[tauri::command]
pub async fn play_playlist(
    playlist_id: String,
    loop_mode: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // Cancel any currently playing playlist
    {
        let playback = state.playlist_playback.lock();
        if let Some(ref pb) = *playback {
            pb.cancel.store(true, Ordering::SeqCst);
        }
    }

    // Small delay to let the previous task notice cancellation
    tokio::time::sleep(std::time::Duration::from_millis(150)).await;

    // Retrieve the playlist data
    let (items, playlist_name) = {
        let playlists = state.playlists.lock();
        let playlist = playlists
            .iter()
            .find(|p| p.id == playlist_id)
            .ok_or_else(|| format!("Playlist not found: {}", playlist_id))?;

        if playlist.items.is_empty() {
            return Err("Playlist is empty".to_string());
        }

        let mut sorted_items = playlist.items.clone();
        sorted_items.sort_by_key(|item| item.order_index);
        (sorted_items, playlist.name.clone())
    };

    let total_items = items.len();
    let cancel = Arc::new(AtomicBool::new(false));

    // Set initial playback state
    {
        let mut playback = state.playlist_playback.lock();
        *playback = Some(PlaylistPlaybackState {
            playlist_id: playlist_id.clone(),
            current_item_index: 0,
            total_items,
            is_playing: true,
            loop_mode,
            cancel: Arc::clone(&cancel),
        });
    }

    tracing::info!(
        "Starting playlist playback: {} ({} items, loop={})",
        playlist_name,
        total_items,
        loop_mode
    );

    // Clone Arc fields from state for the spawned task
    let player_arc = Arc::clone(&state.player);
    let decoded_cache_arc = Arc::clone(&state.decoded_cache);
    let playlist_playback_arc = Arc::clone(&state.playlist_playback);
    let playback_state_arc = Arc::clone(&state.playback_state);

    tokio::spawn(async move {
        let result = playlist_playback_loop(
            &playlist_id,
            &items,
            loop_mode,
            &cancel,
            &player_arc,
            &decoded_cache_arc,
            &playlist_playback_arc,
            &playback_state_arc,
        )
        .await;

        // Clean up playback state when done
        {
            let mut playback = playlist_playback_arc.lock();
            *playback = None;
        }

        match result {
            Ok(()) => tracing::info!("Playlist playback finished"),
            Err(e) => tracing::error!("Playlist playback error: {}", e),
        }
    });

    Ok(())
}

/// Internal loop that drives sequential playlist playback
async fn playlist_playback_loop(
    _playlist_id: &str,
    items: &[crate::types::PlaylistItemInfo],
    loop_mode: bool,
    cancel: &AtomicBool,
    player_arc: &Arc<parking_lot::Mutex<Option<crate::audio::player::AudioPlayer>>>,
    decoded_cache_arc: &Arc<parking_lot::Mutex<std::collections::HashMap<String, crate::audio::decoder::DecodedAudio>>>,
    playlist_playback_arc: &Arc<parking_lot::Mutex<Option<PlaylistPlaybackState>>>,
    playback_state_arc: &Arc<parking_lot::Mutex<crate::types::PlaybackState>>,
) -> Result<(), String> {
    loop {
        for (index, item) in items.iter().enumerate() {
            // Check cancellation
            if cancel.load(Ordering::SeqCst) {
                tracing::info!("Playlist playback cancelled");
                // Stop the player
                let player_lock = player_arc.lock();
                if let Some(ref player) = *player_lock {
                    player.send_command(PlayerCommand::Stop);
                }
                return Ok(());
            }

            // Update current item index in playback state
            {
                let mut playback = playlist_playback_arc.lock();
                if let Some(ref mut pb) = *playback {
                    pb.current_item_index = index;
                    pb.is_playing = true;
                }
            }

            tracing::info!(
                "Playlist item {}/{}: {} (pre={:.2}s, post={:.2}s)",
                index + 1,
                items.len(),
                item.audio_file_name,
                item.pre_silence_sec,
                item.post_silence_sec
            );

            // Pre-silence
            if item.pre_silence_sec > 0.0 {
                let pre_duration =
                    std::time::Duration::from_secs_f32(item.pre_silence_sec);
                if sleep_with_cancel(pre_duration, cancel).await {
                    // Cancelled during pre-silence
                    let player_lock = player_arc.lock();
                    if let Some(ref player) = *player_lock {
                        player.send_command(PlayerCommand::Stop);
                    }
                    return Ok(());
                }
            }

            // Check cancellation again before loading
            if cancel.load(Ordering::SeqCst) {
                return Ok(());
            }

            // Load and play the audio file
            {
                let decoded = {
                    let cache = decoded_cache_arc.lock();
                    cache.get(&item.audio_file_id).cloned()
                };

                let decoded = match decoded {
                    Some(d) => d,
                    None => {
                        tracing::warn!(
                            "Audio file not in decoded cache: {} ({}), skipping",
                            item.audio_file_id,
                            item.audio_file_name
                        );
                        continue;
                    }
                };

                let duration = decoded.metadata.duration_sec;

                let player_lock = player_arc.lock();
                let player = player_lock.as_ref().ok_or_else(|| {
                    "Audio player not available".to_string()
                })?;

                player.load(decoded).map_err(|e| e.to_string())?;

                // Update individual playback state
                {
                    let mut ps = playback_state_arc.lock();
                    ps.file_id = Some(item.audio_file_id.clone());
                    ps.total_duration_sec = duration;
                    ps.current_position_sec = 0.0;
                    ps.is_playing = true;
                    ps.is_paused = false;
                }

                player.send_command(PlayerCommand::Play);
            }

            // Wait for playback to complete by polling player status
            loop {
                if cancel.load(Ordering::SeqCst) {
                    let player_lock = player_arc.lock();
                    if let Some(ref player) = *player_lock {
                        player.send_command(PlayerCommand::Stop);
                    }
                    return Ok(());
                }

                let is_done = {
                    let player_lock = player_arc.lock();
                    if let Some(ref player) = *player_lock {
                        let status = player.get_status();
                        !status.is_playing && !status.is_paused
                    } else {
                        true
                    }
                };

                if is_done {
                    break;
                }

                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }

            // Post-silence
            if item.post_silence_sec > 0.0 {
                let post_duration =
                    std::time::Duration::from_secs_f32(item.post_silence_sec);
                if sleep_with_cancel(post_duration, cancel).await {
                    return Ok(());
                }
            }
        }

        // If not looping, break after one pass
        if !loop_mode {
            break;
        }

        tracing::info!("Playlist loop: restarting from beginning");
    }

    Ok(())
}

/// Sleep for a duration, but check cancellation every 100ms.
/// Returns true if cancelled, false if sleep completed normally.
async fn sleep_with_cancel(duration: std::time::Duration, cancel: &AtomicBool) -> bool {
    let interval = std::time::Duration::from_millis(100);
    let mut remaining = duration;

    while remaining > std::time::Duration::ZERO {
        if cancel.load(Ordering::SeqCst) {
            return true;
        }
        let sleep_time = remaining.min(interval);
        tokio::time::sleep(sleep_time).await;
        remaining = remaining.saturating_sub(sleep_time);
    }

    false
}

/// Get playlist playback status
#[tauri::command]
pub async fn get_playlist_playback_status(
    state: State<'_, AppState>,
) -> Result<Option<PlaylistPlaybackInfo>, String> {
    let playback = state.playlist_playback.lock();
    match *playback {
        Some(ref pb) => {
            // Look up the current item's file name
            let current_file_name = {
                let playlists = state.playlists.lock();
                playlists
                    .iter()
                    .find(|p| p.id == pb.playlist_id)
                    .and_then(|p| {
                        let mut sorted = p.items.clone();
                        sorted.sort_by_key(|i| i.order_index);
                        sorted.get(pb.current_item_index).map(|i| i.audio_file_name.clone())
                    })
                    .unwrap_or_else(|| "unknown".to_string())
            };

            Ok(Some(PlaylistPlaybackInfo {
                playlist_id: pb.playlist_id.clone(),
                current_item_index: pb.current_item_index,
                total_items: pb.total_items,
                current_file_name,
                is_playing: pb.is_playing,
                loop_mode: pb.loop_mode,
            }))
        }
        None => Ok(None),
    }
}

/// Stop playlist playback
#[tauri::command]
pub async fn stop_playlist(state: State<'_, AppState>) -> Result<(), String> {
    {
        let playback = state.playlist_playback.lock();
        if let Some(ref pb) = *playback {
            pb.cancel.store(true, Ordering::SeqCst);
            tracing::info!("Stopping playlist playback: {}", pb.playlist_id);
        } else {
            tracing::info!("No playlist currently playing");
            return Ok(());
        }
    }

    // Stop the underlying audio player as well
    {
        let player_lock = state.player.lock();
        if let Some(ref player) = *player_lock {
            player.send_command(PlayerCommand::Stop);
        }
    }

    // Update individual playback state
    {
        let mut ps = state.playback_state.lock();
        ps.is_playing = false;
        ps.is_paused = false;
        ps.current_position_sec = 0.0;
    }

    // Give the background task a moment to notice cancellation and clean up
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    // Clear playlist playback state (the task should have cleared it, but ensure)
    {
        let mut playback = state.playlist_playback.lock();
        *playback = None;
    }

    Ok(())
}
