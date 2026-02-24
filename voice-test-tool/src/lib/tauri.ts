import { invoke } from "@tauri-apps/api/core";
import type { AudioFileInfo, PlaybackState, WaveformData, PlaylistInfo, PlaylistPlaybackInfo } from "../types/audio";
import type { DeviceInfo, DeviceStatus } from "../types/device";
import type { ScenarioInfo, ScenarioData, ScenarioProgressInfo, TestSessionInfo, LogEntry, ReportFormat } from "../types/scenario";

// ============================================
// Audio file management
// ============================================

/** Load an audio file from the given path */
export async function loadAudioFile(path: string): Promise<AudioFileInfo> {
  return invoke<AudioFileInfo>("load_audio_file", { path });
}

/** Get the list of loaded audio files */
export async function listAudioFiles(): Promise<AudioFileInfo[]> {
  return invoke<AudioFileInfo[]>("list_audio_files");
}

/** Remove an audio file from the loaded list */
export async function removeAudioFile(fileId: string): Promise<void> {
  return invoke<void>("remove_audio_file", { fileId });
}

/** Get waveform data for an audio file */
export async function getWaveformData(
  fileId: string,
  resolution: number
): Promise<WaveformData> {
  const peaks = await invoke<number[]>("get_waveform_data", { fileId, resolution });
  // The backend returns raw peaks; we wrap them in a WaveformData structure
  return {
    peaks,
    duration_sec: 0, // Will be filled by the caller from AudioFileInfo
    samples_per_peak: resolution,
  };
}

// ============================================
// Virtual device management
// ============================================

/** Create a virtual audio device */
export async function createVirtualDevice(name: string): Promise<DeviceInfo> {
  return invoke<DeviceInfo>("create_virtual_device", { name });
}

/** Destroy the active virtual device */
export async function destroyVirtualDevice(): Promise<void> {
  return invoke<void>("destroy_virtual_device");
}

/** List all audio devices */
export async function listAudioDevices(): Promise<DeviceInfo[]> {
  return invoke<DeviceInfo[]>("list_audio_devices");
}

/** Set a device as the default input device */
export async function setDefaultInputDevice(deviceId: string): Promise<void> {
  return invoke<void>("set_default_input_device", { deviceId });
}

/** Get the current device status */
export async function getDeviceStatus(): Promise<DeviceStatus> {
  return invoke<DeviceStatus>("get_device_status");
}

// ============================================
// Playback control
// ============================================

/** Start playback of the given audio file */
export async function play(fileId: string): Promise<void> {
  return invoke<void>("play", { fileId });
}

/** Pause current playback */
export async function pause(): Promise<void> {
  return invoke<void>("pause");
}

/** Stop current playback */
export async function stop(): Promise<void> {
  return invoke<void>("stop");
}

/** Seek to a position in seconds */
export async function seek(positionSec: number): Promise<void> {
  return invoke<void>("seek", { positionSec });
}

/** Set the playback speed */
export async function setPlaybackSpeed(speed: number): Promise<void> {
  return invoke<void>("set_playback_speed", { speed });
}

/** Set the output volume */
export async function setVolume(volume: number): Promise<void> {
  return invoke<void>("set_volume", { volume });
}

/** Get the current playback state */
export async function getPlaybackState(): Promise<PlaybackState> {
  return invoke<PlaybackState>("get_playback_state");
}

/** Audio system status */
export interface AudioSystemStatus {
  available: boolean;
  error: string | null;
}

/** Get the audio system status */
export async function getAudioSystemStatus(): Promise<AudioSystemStatus> {
  return invoke<AudioSystemStatus>("get_audio_system_status");
}

// ============================================
// Playlist management
// ============================================

/** Create a new playlist */
export async function createPlaylist(name: string): Promise<PlaylistInfo> {
  return invoke<PlaylistInfo>("create_playlist", { name });
}

/** Add a file to a playlist */
export async function addToPlaylist(
  playlistId: string,
  fileId: string,
  preSilenceSec: number,
  postSilenceSec: number
): Promise<void> {
  return invoke<void>("add_to_playlist", {
    playlistId,
    fileId,
    preSilenceSec,
    postSilenceSec,
  });
}

/** Play an entire playlist */
export async function playPlaylist(
  playlistId: string,
  loopMode: boolean
): Promise<void> {
  return invoke<void>("play_playlist", { playlistId, loopMode });
}

/** List all playlists */
export async function listPlaylists(): Promise<PlaylistInfo[]> {
  return invoke<PlaylistInfo[]>("list_playlists");
}

/** Get a single playlist by ID */
export async function getPlaylist(playlistId: string): Promise<PlaylistInfo> {
  return invoke<PlaylistInfo>("get_playlist", { playlistId });
}

/** Delete a playlist */
export async function deletePlaylist(playlistId: string): Promise<void> {
  return invoke<void>("delete_playlist", { playlistId });
}

/** Remove an item from a playlist */
export async function removeFromPlaylist(
  playlistId: string,
  itemId: string
): Promise<void> {
  return invoke<void>("remove_from_playlist", { playlistId, itemId });
}

/** Reorder items in a playlist */
export async function reorderPlaylist(
  playlistId: string,
  itemIds: string[]
): Promise<void> {
  return invoke<void>("reorder_playlist", { playlistId, itemIds });
}

/** Update pre/post silence for a playlist item */
export async function updatePlaylistItemSilence(
  playlistId: string,
  itemId: string,
  preSilenceSec: number,
  postSilenceSec: number
): Promise<void> {
  return invoke<void>("update_playlist_item_silence", {
    playlistId,
    itemId,
    preSilenceSec,
    postSilenceSec,
  });
}

/** Stop playlist playback */
export async function stopPlaylist(): Promise<void> {
  return invoke<void>("stop_playlist");
}

/** Get the current playlist playback status */
export async function getPlaylistPlaybackStatus(): Promise<PlaylistPlaybackInfo | null> {
  return invoke<PlaylistPlaybackInfo | null>("get_playlist_playback_status");
}

// ============================================
// Scenario management
// ============================================

/** Load a scenario from a file path */
export async function loadScenario(path: string): Promise<ScenarioInfo> {
  return invoke<ScenarioInfo>("load_scenario", { path });
}

/** Save a scenario to a file path */
export async function saveScenario(
  scenario: ScenarioData,
  path: string
): Promise<void> {
  return invoke<void>("save_scenario", { scenario, path });
}

/** Execute a scenario, returning the session ID */
export async function executeScenario(scenarioId: string): Promise<string> {
  return invoke<string>("execute_scenario", { scenarioId });
}

/** Pause scenario execution */
export async function pauseScenario(sessionId: string): Promise<void> {
  return invoke<void>("pause_scenario", { sessionId });
}

/** Resume scenario execution */
export async function resumeScenario(sessionId: string): Promise<void> {
  return invoke<void>("resume_scenario", { sessionId });
}

/** Abort scenario execution */
export async function abortScenario(sessionId: string): Promise<void> {
  return invoke<void>("abort_scenario", { sessionId });
}

/** List all loaded scenarios */
export async function listScenarios(): Promise<ScenarioInfo[]> {
  return invoke<ScenarioInfo[]>("list_scenarios");
}

/** Delete a scenario by ID */
export async function deleteScenario(scenarioId: string): Promise<void> {
  return invoke<void>("delete_scenario", { scenarioId });
}

/** Get the execution status for a scenario session */
export async function getScenarioExecutionStatus(sessionId: string): Promise<ScenarioProgressInfo | null> {
  return invoke<ScenarioProgressInfo | null>("get_scenario_execution_status", { sessionId });
}

// ============================================
// Logs & reports
// ============================================

/** Get logs for a test session */
export async function getSessionLogs(sessionId: string): Promise<LogEntry[]> {
  return invoke<LogEntry[]>("get_session_logs", { sessionId });
}

/** Export a report for a test session */
export async function exportReport(
  sessionId: string,
  format: ReportFormat,
  outputPath: string
): Promise<string> {
  return invoke<string>("export_report", { sessionId, format, outputPath });
}

// ============================================
// Test sessions
// ============================================

/** List all test sessions */
export async function listTestSessions(): Promise<TestSessionInfo[]> {
  return invoke<TestSessionInfo[]>("list_test_sessions");
}

/** Get a single test session by ID */
export async function getTestSession(sessionId: string): Promise<TestSessionInfo> {
  return invoke<TestSessionInfo>("get_test_session", { sessionId });
}

// ============================================
// Configuration
// ============================================

/** TTS Model options */
export type TtsModel = "Tts1" | "Tts1Hd" | "Gpt4oMiniTts";

/** TTS Voice options */
export type TtsVoice =
  | "Alloy"
  | "Ash"
  | "Ballad"
  | "Coral"
  | "Echo"
  | "Fable"
  | "Nova"
  | "Onyx"
  | "Sage"
  | "Shimmer"
  | "Verse";

/** TTS Output Format options */
export type TtsOutputFormat = "Mp3" | "Opus" | "Aac" | "Flac" | "Wav" | "Pcm";

/** TTS configuration matching the Rust TtsConfig struct */
export interface TtsConfig {
  api_key: string | null;
  base_url: string | null;
  model: TtsModel;
  default_voice: TtsVoice;
  default_speed: number;
  default_instructions: string | null;
  output_format: TtsOutputFormat;
}

/** Application configuration matching the Rust AppConfig struct */
export interface AppConfig {
  audio: {
    default_sample_rate: number;
    default_channels: number;
    buffer_size: number;
    default_playback_speed: number;
    default_volume: number;
  };
  virtual_device: {
    device_name: string;
    auto_create_on_startup: boolean;
    set_as_default: boolean;
    cleanup_on_exit: boolean;
  };
  ui: {
    theme: string;
    language: string;
    window_width: number;
    window_height: number;
    waveform_color: string;
    waveform_progress_color: string;
  };
  logging: {
    level: string;
    output_dir: string;
    max_sessions: number;
  };
  shortcuts: {
    play_pause: string;
    stop: string;
    next: string;
    previous: string;
    seek_forward: string;
    seek_backward: string;
  };
  tts: TtsConfig;
}

/** Get the current application configuration */
export async function getConfig(): Promise<AppConfig> {
  return invoke<AppConfig>("get_config");
}

/** Update the application configuration */
export async function updateConfig(config: AppConfig): Promise<void> {
  return invoke<void>("update_config", { config });
}

// ============================================
// TTS generation
// ============================================

/** TTS test case from CSV */
export interface TtsTestCase {
  /** Unique identifier (e.g., "001", "002") */
  id: string;
  /** Text to be converted to speech */
  text: string;
}

/** Load TTS test cases from a CSV file */
export async function loadTtsCsv(path: string): Promise<TtsTestCase[]> {
  return invoke<TtsTestCase[]>("load_tts_csv", { path });
}

/** Generate TTS audio for preview */
export async function previewTts(text: string): Promise<Uint8Array> {
  const bytes = await invoke<number[]>("preview_tts", { text });
  return new Uint8Array(bytes);
}

// ============================================
// TTS Test Execution
// ============================================

/** Result of a single TTS test case execution */
export interface TtsTestResult {
  test_case_id: string;
  input_text: string;
  tts_duration_sec: number;
  response_duration_sec: number | null;
  started_at: string;
  ended_at: string;
  error: string | null;
}

/** Status of a TTS test session */
export type TtsTestStatus =
  | { type: "Pending" }
  | { type: "Running"; current_index: number }
  | { type: "Paused"; current_index: number }
  | { type: "Completed" }
  | { type: "Aborted" }
  | { type: "Failed"; message: string };

/** A TTS test session */
export interface TtsTestSession {
  id: string;
  csv_path: string;
  test_cases: TtsTestCase[];
  results: TtsTestResult[];
  config: TtsConfig;
  recording_config: RecordingConfig;
  status: TtsTestStatus;
  started_at: string;
  ended_at: string | null;
}

/** Start a TTS test execution */
export async function startTtsTest(csvPath: string): Promise<string> {
  return invoke<string>("start_tts_test", { csvPath });
}

/** Pause a running TTS test */
export async function pauseTtsTest(sessionId: string): Promise<void> {
  return invoke<void>("pause_tts_test", { sessionId });
}

/** Resume a paused TTS test */
export async function resumeTtsTest(sessionId: string): Promise<void> {
  return invoke<void>("resume_tts_test", { sessionId });
}

/** Abort a running TTS test */
export async function abortTtsTest(sessionId: string): Promise<void> {
  return invoke<void>("abort_tts_test", { sessionId });
}

/** Get the status of a TTS test session */
export async function getTtsTestStatus(sessionId: string): Promise<TtsTestStatus> {
  return invoke<TtsTestStatus>("get_tts_test_status", { sessionId });
}

/** Get the full session data for a TTS test */
export async function getTtsTestSession(sessionId: string): Promise<TtsTestSession> {
  return invoke<TtsTestSession>("get_tts_test_session", { sessionId });
}

/** Export a TTS test session to a ZIP archive */
export async function exportTtsTest(
  sessionId: string,
  outputPath: string
): Promise<string> {
  return invoke<string>("export_tts_test", {
    sessionId,
    outputPath,
  });
}

// ============================================
// Recording
// ============================================

/** Recording configuration */
export interface RecordingConfig {
  device_name: string | null;
  sample_rate: number;
  channels: number;
  silence_threshold_db: number;
  silence_duration_ms: number;
  max_recording_sec: number;
}

/** Audio device information */
export interface InputDeviceInfo {
  name: string;
  is_default: boolean;
  /** Device type: "input" for microphones, "output" for speakers/loopback */
  device_type: "input" | "output";
}

/** List available input devices */
export async function listInputDevices(): Promise<InputDeviceInfo[]> {
  return invoke<InputDeviceInfo[]>("list_input_devices");
}

/** Get the current recording configuration */
export async function getRecordingConfig(): Promise<RecordingConfig> {
  return invoke<RecordingConfig>("get_recording_config");
}

/** Update the recording configuration */
export async function updateRecordingConfig(config: RecordingConfig): Promise<void> {
  return invoke<void>("update_recording_config", { config });
}
