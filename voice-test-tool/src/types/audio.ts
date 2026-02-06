/** Audio file format type */
export type AudioFormat = "wav" | "mp3" | "flac" | "ogg";

/**
 * Information about a loaded audio file, matching the Rust AudioFileInfo struct.
 *
 * NOTE: The Rust struct does NOT include `created_at`. The `format` field is
 * returned as a plain `string` from serde but we keep the narrow type for
 * UI convenience and accept `string` at the boundary via the type alias below.
 */
export interface AudioFileInfo {
  id: string;
  path: string;
  name: string;
  sample_rate: number;
  channels: number;
  duration_sec: number;
  format: string;
}

/**
 * Current playback state, matching the Rust PlaybackState struct.
 *
 * NOTE: The Rust struct does NOT include `loop_enabled` - that is frontend-only
 * state managed by the playback store.
 */
export interface PlaybackState {
  file_id: string | null;
  is_playing: boolean;
  is_paused: boolean;
  current_position_sec: number;
  total_duration_sec: number;
  volume: number;
  speed: number;
  /** Frontend-only field - not returned by the Rust backend */
  loop_enabled: boolean;
}

/** Waveform data for rendering */
export interface WaveformData {
  /** Normalized amplitude peaks (0.0 to 1.0) */
  peaks: number[];
  /** Duration of the audio in seconds */
  duration_sec: number;
  /** Number of samples per peak */
  samples_per_peak: number;
}

/** Playlist information */
export interface PlaylistInfo {
  id: string;
  name: string;
  items: PlaylistItemInfo[];
  created_at: string;
  updated_at: string;
}

/** Single item in a playlist */
export interface PlaylistItemInfo {
  id: string;
  playlist_id: string;
  audio_file_id: string;
  order_index: number;
  pre_silence_sec: number;
  post_silence_sec: number;
}

/** Playlist playback status from the backend */
export interface PlaylistPlaybackInfo {
  playlist_id: string;
  current_item_index: number;
  total_items: number;
  current_file_name: string;
  is_playing: boolean;
  loop_mode: boolean;
}
