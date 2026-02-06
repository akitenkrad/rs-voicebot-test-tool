/** Playback progress event emitted by the Rust backend */
export interface PlaybackProgressEvent {
  file_id: string;
  current_position_sec: number;
  total_duration_sec: number;
  is_playing: boolean;
}

/** Device status change event emitted by the Rust backend */
export interface DeviceStatusEvent {
  device_name: string;
  is_active: boolean;
  is_default: boolean;
  error?: string;
}

/** Scenario execution progress event emitted by the Rust backend */
export interface ScenarioProgressEvent {
  session_id: string;
  current_turn: number;
  total_turns: number;
  status: "running" | "paused" | "completed" | "failed" | "aborted";
  current_file_name?: string;
}

/** Log event emitted by the Rust backend */
export interface LogEvent {
  session_id: string;
  timestamp: string;
  level: "info" | "warn" | "error";
  message: string;
  details?: Record<string, unknown>;
}

/** All event type names used for Tauri event listeners */
export const TAURI_EVENTS = {
  PLAYBACK_PROGRESS: "playback-progress",
  DEVICE_STATUS: "device-status",
  SCENARIO_PROGRESS: "scenario-progress",
  LOG_EVENT: "log-event",
} as const;

export type TauriEventName =
  (typeof TAURI_EVENTS)[keyof typeof TAURI_EVENTS];
