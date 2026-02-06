/** Basic information about a loaded scenario */
export interface ScenarioInfo {
  id: string;
  name: string;
  description: string;
  turn_count: number;
  created_at: string;
  updated_at: string;
}

/** A single turn within a scenario */
export interface ScenarioTurnInfo {
  id: string;
  audio_file: string;
  order_index: number;
  delay_before_ms: number;
  delay_after_ms: number;
  expected_transcript?: string;
  expected_response?: string;
  conditions?: Record<string, unknown>;
}

/** Full scenario data including turns */
export interface ScenarioData {
  name: string;
  description: string;
  metadata: ScenarioMetadata;
  turns: ScenarioTurnInfo[];
}

/** Scenario metadata */
export interface ScenarioMetadata {
  author?: string;
  version?: string;
  created_at?: string;
  [key: string]: unknown;
}

/** Execution status of a scenario */
export type ScenarioExecutionStatus =
  | "idle"
  | "running"
  | "paused"
  | "completed"
  | "failed"
  | "aborted";

/** Test session representing a single scenario execution */
export interface TestSession {
  id: string;
  scenario_id: string;
  started_at: string;
  ended_at?: string;
  status: ScenarioExecutionStatus;
  current_turn: number;
  total_turns: number;
  result_summary?: Record<string, unknown>;
}

/** Log entry level */
export type LogLevel = "info" | "warn" | "error";

/** A single log entry from a test session */
export interface LogEntry {
  id: string;
  session_id: string;
  timestamp: string;
  level: LogLevel;
  message: string;
  details?: Record<string, unknown>;
}

/** Progress information for an executing scenario */
export interface ScenarioProgressInfo {
  session_id: string;
  scenario_id: string;
  current_turn: number;
  total_turns: number;
  status: 'running' | 'paused' | 'completed' | 'failed' | 'aborted';
  current_file_name: string | null;
}

/** Information about a test session */
export interface TestSessionInfo {
  id: string;
  scenario_id: string;
  scenario_name: string;
  started_at: string;
  ended_at: string | null;
  status: string;
}

/** Report export format */
export type ReportFormat = "html" | "json" | "csv";
