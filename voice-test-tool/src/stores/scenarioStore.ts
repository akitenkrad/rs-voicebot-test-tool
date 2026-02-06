import { create } from "zustand";
import { immer } from "zustand/middleware/immer";
import type { ScenarioInfo, ScenarioProgressInfo, TestSessionInfo, LogEntry } from "../types/scenario";

export interface ScenarioState {
  /** List of loaded scenarios */
  scenarios: ScenarioInfo[];
  /** Currently selected scenario ID */
  selectedScenarioId: string | null;
  /** Active execution session ID */
  activeSessionId: string | null;
  /** Current execution status (progress info) */
  executionStatus: ScenarioProgressInfo | null;
  /** List of past test sessions */
  testSessions: TestSessionInfo[];
  /** Log entries for the viewed session */
  logs: LogEntry[];
  /** Whether a scenario operation is in progress */
  isLoading: boolean;
  /** Error message if scenario operation failed */
  error: string | null;
}

export interface ScenarioActions {
  /** Set the list of scenarios */
  setScenarios: (scenarios: ScenarioInfo[]) => void;
  /** Add a scenario to the list */
  addScenario: (scenario: ScenarioInfo) => void;
  /** Remove a scenario by ID */
  removeScenario: (scenarioId: string) => void;
  /** Set the selected scenario ID */
  setSelectedScenarioId: (scenarioId: string | null) => void;
  /** Set the active session ID */
  setActiveSessionId: (sessionId: string | null) => void;
  /** Set the execution status */
  setExecutionStatus: (status: ScenarioProgressInfo | null) => void;
  /** Set test sessions list */
  setTestSessions: (sessions: TestSessionInfo[]) => void;
  /** Add a log entry */
  addLog: (log: LogEntry) => void;
  /** Set all logs (e.g., when loading session logs) */
  setLogs: (logs: LogEntry[]) => void;
  /** Clear logs */
  clearLogs: () => void;
  /** Set loading state */
  setLoading: (isLoading: boolean) => void;
  /** Set error message */
  setError: (error: string | null) => void;
}

export type ScenarioStore = ScenarioState & ScenarioActions;

export const useScenarioStore = create<ScenarioStore>()(
  immer((set) => ({
    scenarios: [],
    selectedScenarioId: null,
    activeSessionId: null,
    executionStatus: null,
    testSessions: [],
    logs: [],
    isLoading: false,
    error: null,

    setScenarios: (scenarios) =>
      set((state) => {
        state.scenarios = scenarios;
      }),

    addScenario: (scenario) =>
      set((state) => {
        state.scenarios.push(scenario);
      }),

    removeScenario: (scenarioId) =>
      set((state) => {
        state.scenarios = state.scenarios.filter((s) => s.id !== scenarioId);
        if (state.selectedScenarioId === scenarioId) {
          state.selectedScenarioId = null;
        }
      }),

    setSelectedScenarioId: (scenarioId) =>
      set((state) => {
        state.selectedScenarioId = scenarioId;
      }),

    setActiveSessionId: (sessionId) =>
      set((state) => {
        state.activeSessionId = sessionId;
      }),

    setExecutionStatus: (status) =>
      set((state) => {
        state.executionStatus = status;
      }),

    setTestSessions: (sessions) =>
      set((state) => {
        state.testSessions = sessions;
      }),

    addLog: (log) =>
      set((state) => {
        state.logs.push(log);
      }),

    setLogs: (logs) =>
      set((state) => {
        state.logs = logs;
      }),

    clearLogs: () =>
      set((state) => {
        state.logs = [];
      }),

    setLoading: (isLoading) =>
      set((state) => {
        state.isLoading = isLoading;
      }),

    setError: (error) =>
      set((state) => {
        state.error = error;
      }),
  }))
);
