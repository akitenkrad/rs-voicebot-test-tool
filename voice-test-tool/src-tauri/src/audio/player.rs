//! Audio playback engine
//!
//! Provides audio playback using rodio with command-based control.
//! The player runs on a separate thread to avoid blocking the Tauri async runtime.

use std::sync::Arc;
use std::time::Instant;

use crossbeam_channel::{Receiver, Sender};
use parking_lot::Mutex;
use rodio::{OutputStream, OutputStreamHandle, Sink};

use crate::error::AudioError;

use super::decoder::DecodedAudio;

/// Commands sent to the audio player thread
#[derive(Debug)]
pub enum PlayerCommand {
    /// Start or resume playback
    Play,
    /// Pause playback
    Pause,
    /// Stop playback and reset position
    Stop,
    /// Seek to position in seconds
    Seek(f64),
    /// Set volume (0.0 - 1.0)
    SetVolume(f32),
    /// Set playback speed (0.5 - 2.0)
    SetSpeed(f32),
    /// Load new audio data
    Load(Arc<DecodedAudio>),
    /// Shutdown the player thread
    Shutdown,
}

/// Current status of the audio player
#[derive(Debug, Clone)]
pub struct PlayerStatus {
    pub is_playing: bool,
    pub is_paused: bool,
    pub position_sec: f64,
    pub duration_sec: f64,
}

impl Default for PlayerStatus {
    fn default() -> Self {
        Self {
            is_playing: false,
            is_paused: false,
            position_sec: 0.0,
            duration_sec: 0.0,
        }
    }
}

/// Shared state between the player thread and the main thread
struct SharedState {
    status: PlayerStatus,
    volume: f32,
    speed: f32,
}

/// Audio player that manages playback on a dedicated thread
pub struct AudioPlayer {
    command_tx: Sender<PlayerCommand>,
    shared: Arc<Mutex<SharedState>>,
    _thread_handle: Option<std::thread::JoinHandle<()>>,
}

impl AudioPlayer {
    /// Create a new AudioPlayer.
    ///
    /// This spawns a dedicated audio thread that handles rodio playback.
    /// The default output device is used initially.
    pub fn new() -> Result<Self, AudioError> {
        let (command_tx, command_rx) = crossbeam_channel::unbounded::<PlayerCommand>();

        let shared = Arc::new(Mutex::new(SharedState {
            status: PlayerStatus::default(),
            volume: 0.8,
            speed: 1.0,
        }));

        let shared_clone = Arc::clone(&shared);

        let handle = std::thread::Builder::new()
            .name("audio-player".to_string())
            .spawn(move || {
                player_thread(command_rx, shared_clone);
            })
            .map_err(|e| AudioError::PlaybackFailed(format!("Failed to spawn audio thread: {}", e)))?;

        Ok(Self {
            command_tx,
            shared,
            _thread_handle: Some(handle),
        })
    }

    /// Load decoded audio into the player for playback
    pub fn load(&self, audio: DecodedAudio) -> Result<(), AudioError> {
        let audio_arc = Arc::new(audio);
        {
            let mut state = self.shared.lock();
            state.status.duration_sec = audio_arc.metadata.duration_sec;
            state.status.position_sec = 0.0;
            state.status.is_playing = false;
            state.status.is_paused = false;
        }
        self.command_tx
            .send(PlayerCommand::Load(audio_arc))
            .map_err(|e| AudioError::PlaybackFailed(format!("Failed to send load command: {}", e)))?;
        Ok(())
    }

    /// Send a command to the player
    pub fn send_command(&self, cmd: PlayerCommand) {
        if let Err(e) = self.command_tx.send(cmd) {
            tracing::error!("Failed to send player command: {}", e);
        }
    }

    /// Get the current player status
    pub fn get_status(&self) -> PlayerStatus {
        self.shared.lock().status.clone()
    }

    /// Set the output device by name.
    ///
    /// If `device_name` is None, the default output device is used.
    /// Note: Changing output device requires recreating the audio stream,
    /// which will interrupt any current playback.
    pub fn set_output_device(&self, _device_name: Option<String>) -> Result<(), AudioError> {
        // rodio 0.19 uses the default output device. Custom device selection
        // would require using cpal directly. For now, we log and use default.
        // This can be extended later to enumerate cpal devices and select one.
        tracing::info!("Output device selection is currently using system default");
        Ok(())
    }
}

impl Drop for AudioPlayer {
    fn drop(&mut self) {
        let _ = self.command_tx.send(PlayerCommand::Shutdown);
        if let Some(handle) = self._thread_handle.take() {
            let _ = handle.join();
        }
    }
}

/// A rodio Source that plays from decoded f32 samples in memory.
///
/// This wraps the decoded audio data and provides an iterator over samples,
/// supporting seek by resetting the position cursor.
struct MemorySource {
    data: Arc<DecodedAudio>,
    position: usize,
}

impl MemorySource {
    fn new(data: Arc<DecodedAudio>) -> Self {
        Self { data, position: 0 }
    }

    fn seek_to_sample(&mut self, sample_index: usize) {
        let channels = self.data.metadata.channels as usize;
        // Align to frame boundary
        let aligned = (sample_index / channels) * channels;
        self.position = aligned.min(self.data.samples.len());
    }

    fn current_position_sec(&self) -> f64 {
        let channels = self.data.metadata.channels.max(1) as f64;
        let sample_rate = self.data.metadata.sample_rate as f64;
        (self.position as f64 / channels) / sample_rate
    }
}

impl Iterator for MemorySource {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        if self.position < self.data.samples.len() {
            let sample = self.data.samples[self.position];
            self.position += 1;
            Some(sample)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.data.samples.len() - self.position;
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for MemorySource {}

impl rodio::Source for MemorySource {
    fn current_frame_len(&self) -> Option<usize> {
        Some(self.data.samples.len() - self.position)
    }

    fn channels(&self) -> u16 {
        self.data.metadata.channels.max(1)
    }

    fn sample_rate(&self) -> u32 {
        self.data.metadata.sample_rate
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        Some(std::time::Duration::from_secs_f64(
            self.data.metadata.duration_sec,
        ))
    }
}

/// The main player thread function.
///
/// This runs in a dedicated thread and manages the rodio OutputStream and Sink.
/// It processes commands from the channel and updates shared state.
fn player_thread(command_rx: Receiver<PlayerCommand>, shared: Arc<Mutex<SharedState>>) {
    // Try to create the output stream. If it fails (no audio device),
    // we still accept commands but log errors on playback attempts.
    let stream_result = OutputStream::try_default();

    let (_stream, stream_handle): (Option<OutputStream>, Option<OutputStreamHandle>) =
        match stream_result {
            Ok((stream, handle)) => {
                tracing::info!("Audio output stream created successfully");
                (Some(stream), Some(handle))
            }
            Err(e) => {
                tracing::warn!("No audio output device available: {}. Playback will not work.", e);
                (None, None)
            }
        };

    let mut current_audio: Option<Arc<DecodedAudio>> = None;
    let mut sink: Option<Sink> = None;
    let mut playback_start_time: Option<Instant> = None;
    let mut playback_offset_sec: f64 = 0.0;
    let mut current_speed: f32 = 1.0;

    // Helper: create a new sink and load audio
    let create_sink_and_play =
        |handle: &OutputStreamHandle,
         audio: &Arc<DecodedAudio>,
         volume: f32,
         speed: f32,
         start_sec: f64|
         -> Result<Sink, AudioError> {
            let sink = Sink::try_new(handle)
                .map_err(|e| AudioError::PlaybackFailed(format!("Failed to create sink: {}", e)))?;

            let mut source = MemorySource::new(Arc::clone(audio));

            // Seek to start position
            if start_sec > 0.0 {
                let channels = audio.metadata.channels.max(1) as usize;
                let sample_idx =
                    (start_sec * audio.metadata.sample_rate as f64) as usize * channels;
                source.seek_to_sample(sample_idx);
            }

            sink.set_volume(volume);
            sink.set_speed(speed);
            sink.append(source);

            Ok(sink)
        };

    loop {
        // Use a timeout to periodically update position
        match command_rx.recv_timeout(std::time::Duration::from_millis(50)) {
            Ok(PlayerCommand::Load(audio)) => {
                tracing::info!(
                    "Loading audio: {:.2}s, {}Hz, {} ch",
                    audio.metadata.duration_sec,
                    audio.metadata.sample_rate,
                    audio.metadata.channels
                );

                // Stop any current playback
                if let Some(ref s) = sink {
                    s.stop();
                }
                sink = None;
                playback_start_time = None;
                playback_offset_sec = 0.0;

                current_audio = Some(audio);

                let mut state = shared.lock();
                state.status.is_playing = false;
                state.status.is_paused = false;
                state.status.position_sec = 0.0;
            }

            Ok(PlayerCommand::Play) => {
                if let (Some(ref handle), Some(ref audio)) = (&stream_handle, &current_audio) {
                    // If we have an existing paused sink, unpause it
                    if let Some(ref s) = sink {
                        if s.is_paused() {
                            s.play();
                            playback_start_time = Some(Instant::now());
                            let mut state = shared.lock();
                            state.status.is_playing = true;
                            state.status.is_paused = false;
                            tracing::info!("Playback resumed");
                            continue;
                        }
                    }

                    // Create a new sink and start playing
                    let volume = shared.lock().volume;
                    let speed = shared.lock().speed;
                    current_speed = speed;

                    match create_sink_and_play(handle, audio, volume, speed, playback_offset_sec) {
                        Ok(new_sink) => {
                            sink = Some(new_sink);
                            playback_start_time = Some(Instant::now());
                            let mut state = shared.lock();
                            state.status.is_playing = true;
                            state.status.is_paused = false;
                            tracing::info!("Playback started at {:.2}s", playback_offset_sec);
                        }
                        Err(e) => {
                            tracing::error!("Failed to start playback: {}", e);
                        }
                    }
                } else if stream_handle.is_none() {
                    tracing::error!("Cannot play: no audio output device available");
                } else {
                    tracing::warn!("Cannot play: no audio loaded");
                }
            }

            Ok(PlayerCommand::Pause) => {
                if let Some(ref s) = sink {
                    s.pause();

                    // Calculate current position before pausing
                    if let Some(start) = playback_start_time.take() {
                        let elapsed = start.elapsed().as_secs_f64() * current_speed as f64;
                        playback_offset_sec += elapsed;
                    }

                    let mut state = shared.lock();
                    state.status.is_playing = false;
                    state.status.is_paused = true;
                    state.status.position_sec = playback_offset_sec;
                    tracing::info!("Playback paused at {:.2}s", playback_offset_sec);
                }
            }

            Ok(PlayerCommand::Stop) => {
                if let Some(ref s) = sink {
                    s.stop();
                }
                sink = None;
                playback_start_time = None;
                playback_offset_sec = 0.0;

                let mut state = shared.lock();
                state.status.is_playing = false;
                state.status.is_paused = false;
                state.status.position_sec = 0.0;
                tracing::info!("Playback stopped");
            }

            Ok(PlayerCommand::Seek(position_sec)) => {
                let position_sec = position_sec.max(0.0);

                if let Some(ref audio) = current_audio {
                    let clamped = position_sec.min(audio.metadata.duration_sec);
                    playback_offset_sec = clamped;

                    let was_playing = {
                        let state = shared.lock();
                        state.status.is_playing
                    };

                    // Stop current sink and recreate at new position
                    if let Some(ref s) = sink {
                        s.stop();
                    }
                    sink = None;

                    if was_playing {
                        if let Some(ref handle) = stream_handle {
                            let volume = shared.lock().volume;
                            let speed = shared.lock().speed;
                            current_speed = speed;

                            match create_sink_and_play(handle, audio, volume, speed, clamped) {
                                Ok(new_sink) => {
                                    sink = Some(new_sink);
                                    playback_start_time = Some(Instant::now());
                                }
                                Err(e) => {
                                    tracing::error!("Failed to seek and play: {}", e);
                                }
                            }
                        }
                    } else {
                        playback_start_time = None;
                    }

                    let mut state = shared.lock();
                    state.status.position_sec = clamped;
                    tracing::info!("Seeked to {:.2}s", clamped);
                }
            }

            Ok(PlayerCommand::SetVolume(volume)) => {
                let volume = volume.clamp(0.0, 1.0);
                if let Some(ref s) = sink {
                    s.set_volume(volume);
                }
                shared.lock().volume = volume;
                tracing::info!("Volume set to {:.2}", volume);
            }

            Ok(PlayerCommand::SetSpeed(speed)) => {
                let speed = speed.clamp(0.5, 2.0);

                // When changing speed during playback, we need to recalculate
                // the position offset to keep position tracking accurate
                if let Some(start) = playback_start_time.take() {
                    let elapsed = start.elapsed().as_secs_f64() * current_speed as f64;
                    playback_offset_sec += elapsed;
                    playback_start_time = Some(Instant::now());
                }

                current_speed = speed;
                if let Some(ref s) = sink {
                    s.set_speed(speed);
                }
                shared.lock().speed = speed;
                tracing::info!("Speed set to {:.2}x", speed);
            }

            Ok(PlayerCommand::Shutdown) => {
                tracing::info!("Audio player shutting down");
                if let Some(ref s) = sink {
                    s.stop();
                }
                break;
            }

            Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                // Periodic update - no command received
            }

            Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {
                tracing::info!("Audio player command channel disconnected, shutting down");
                break;
            }
        }

        // Update playback position
        let mut state = shared.lock();
        if state.status.is_playing {
            if let Some(start) = playback_start_time {
                let elapsed = start.elapsed().as_secs_f64() * current_speed as f64;
                let current_pos = playback_offset_sec + elapsed;
                state.status.position_sec = current_pos;

                // Check if playback has finished
                if current_pos >= state.status.duration_sec {
                    state.status.is_playing = false;
                    state.status.is_paused = false;
                    state.status.position_sec = state.status.duration_sec;

                    // Drop the playback start time since we're done
                    drop(state);
                    playback_start_time = None;
                    playback_offset_sec = 0.0;

                    if let Some(ref s) = sink {
                        s.stop();
                    }
                    sink = None;

                    tracing::info!("Playback finished");
                }
            }
        }
    }
}
