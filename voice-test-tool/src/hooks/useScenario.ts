import { useCallback, useEffect, useRef } from "react";
import { useScenarioStore } from "../stores/scenarioStore";
import * as tauri from "../lib/tauri";
import type { ScenarioData, ReportFormat } from "../types/scenario";

/** Polling interval for execution status (ms) */
const STATUS_POLL_INTERVAL = 500;

/** Polling interval for logs while running (ms) */
const LOG_POLL_INTERVAL = 1000;

/**
 * Hook for scenario management.
 * Combines scenarioStore state with Tauri command invocations.
 */
export function useScenario() {
  const store = useScenarioStore();
  const statusPollRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const logPollRef = useRef<ReturnType<typeof setInterval> | null>(null);

  // Determine if execution is active (running or paused)
  const isExecutionActive =
    store.executionStatus?.status === "running" ||
    store.executionStatus?.status === "paused";

  // -- Polling: execution status --
  const refreshExecutionStatus = useCallback(async () => {
    const sessionId = useScenarioStore.getState().activeSessionId;
    if (!sessionId) return;
    try {
      const status = await tauri.getScenarioExecutionStatus(sessionId);
      store.setExecutionStatus(status);
      // If execution finished, clear active session
      if (
        status &&
        (status.status === "completed" ||
          status.status === "failed" ||
          status.status === "aborted")
      ) {
        // Keep executionStatus for display but stop polling
      }
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      store.setError(message);
    }
  }, [store]);

  // -- Polling: logs --
  const fetchLogs = useCallback(
    async (sessionId: string) => {
      try {
        const logs = await tauri.getSessionLogs(sessionId);
        store.setLogs(logs);
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        store.setError(message);
      }
    },
    [store]
  );

  // Auto-poll execution status while active
  useEffect(() => {
    if (isExecutionActive) {
      if (statusPollRef.current === null) {
        statusPollRef.current = setInterval(() => {
          refreshExecutionStatus();
        }, STATUS_POLL_INTERVAL);
      }
    } else {
      if (statusPollRef.current !== null) {
        clearInterval(statusPollRef.current);
        statusPollRef.current = null;
      }
    }
    return () => {
      if (statusPollRef.current !== null) {
        clearInterval(statusPollRef.current);
        statusPollRef.current = null;
      }
    };
  }, [isExecutionActive, refreshExecutionStatus]);

  // Auto-poll logs while execution is active
  useEffect(() => {
    const sessionId = store.activeSessionId;
    if (isExecutionActive && sessionId) {
      if (logPollRef.current === null) {
        logPollRef.current = setInterval(() => {
          fetchLogs(sessionId);
        }, LOG_POLL_INTERVAL);
      }
    } else {
      if (logPollRef.current !== null) {
        clearInterval(logPollRef.current);
        logPollRef.current = null;
      }
    }
    return () => {
      if (logPollRef.current !== null) {
        clearInterval(logPollRef.current);
        logPollRef.current = null;
      }
    };
  }, [isExecutionActive, store.activeSessionId, fetchLogs]);

  // -- Actions --

  const fetchScenarios = useCallback(async () => {
    store.setLoading(true);
    store.setError(null);
    try {
      const scenarios = await tauri.listScenarios();
      store.setScenarios(scenarios);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      store.setError(message);
    } finally {
      store.setLoading(false);
    }
  }, [store]);

  const loadScenarioFile = useCallback(
    async (path: string) => {
      store.setLoading(true);
      store.setError(null);
      try {
        const scenario = await tauri.loadScenario(path);
        store.addScenario(scenario);
        return scenario;
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        store.setError(message);
        throw err;
      } finally {
        store.setLoading(false);
      }
    },
    [store]
  );

  const saveScenario = useCallback(
    async (scenario: ScenarioData, path: string) => {
      store.setLoading(true);
      store.setError(null);
      try {
        await tauri.saveScenario(scenario, path);
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        store.setError(message);
        throw err;
      } finally {
        store.setLoading(false);
      }
    },
    [store]
  );

  const deleteScenarioById = useCallback(
    async (scenarioId: string) => {
      store.setError(null);
      try {
        await tauri.deleteScenario(scenarioId);
        store.removeScenario(scenarioId);
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        store.setError(message);
        throw err;
      }
    },
    [store]
  );

  const executeScenario = useCallback(
    async (scenarioId: string) => {
      store.setLoading(true);
      store.setError(null);
      try {
        const sessionId = await tauri.executeScenario(scenarioId);
        store.setActiveSessionId(sessionId);
        store.setExecutionStatus({
          session_id: sessionId,
          scenario_id: scenarioId,
          current_turn: 0,
          total_turns: 0,
          status: "running",
          current_file_name: null,
        });
        store.clearLogs();
        return sessionId;
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        store.setError(message);
        throw err;
      } finally {
        store.setLoading(false);
      }
    },
    [store]
  );

  const pauseExecution = useCallback(async () => {
    const sessionId = useScenarioStore.getState().activeSessionId;
    if (!sessionId) return;
    try {
      await tauri.pauseScenario(sessionId);
      const current = useScenarioStore.getState().executionStatus;
      if (current) {
        store.setExecutionStatus({ ...current, status: "paused" });
      }
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      store.setError(message);
      throw err;
    }
  }, [store]);

  const resumeExecution = useCallback(async () => {
    const sessionId = useScenarioStore.getState().activeSessionId;
    if (!sessionId) return;
    try {
      await tauri.resumeScenario(sessionId);
      const current = useScenarioStore.getState().executionStatus;
      if (current) {
        store.setExecutionStatus({ ...current, status: "running" });
      }
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      store.setError(message);
      throw err;
    }
  }, [store]);

  const abortExecution = useCallback(async () => {
    const sessionId = useScenarioStore.getState().activeSessionId;
    if (!sessionId) return;
    try {
      await tauri.abortScenario(sessionId);
      const current = useScenarioStore.getState().executionStatus;
      if (current) {
        store.setExecutionStatus({ ...current, status: "aborted" });
      }
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      store.setError(message);
      throw err;
    }
  }, [store]);

  const fetchTestSessions = useCallback(async () => {
    try {
      const sessions = await tauri.listTestSessions();
      store.setTestSessions(sessions);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      store.setError(message);
    }
  }, [store]);

  const exportReport = useCallback(
    async (sessionId: string, format: ReportFormat, outputPath: string) => {
      store.setError(null);
      try {
        const result = await tauri.exportReport(sessionId, format, outputPath);
        return result;
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        store.setError(message);
        throw err;
      }
    },
    [store]
  );

  // Derive the selected scenario from the store
  const selectedScenario =
    store.scenarios.find((s) => s.id === store.selectedScenarioId) ?? null;

  return {
    // State
    scenarios: store.scenarios,
    selectedScenarioId: store.selectedScenarioId,
    selectedScenario,
    executionStatus: store.executionStatus,
    activeSessionId: store.activeSessionId,
    testSessions: store.testSessions,
    logs: store.logs,
    isLoading: store.isLoading,
    error: store.error,
    // Actions
    fetchScenarios,
    loadScenarioFile,
    saveScenario,
    deleteScenario: deleteScenarioById,
    executeScenario,
    pauseExecution,
    resumeExecution,
    abortExecution,
    refreshExecutionStatus,
    fetchLogs,
    fetchTestSessions,
    exportReport,
    setSelectedScenarioId: store.setSelectedScenarioId,
    clearLogs: store.clearLogs,
  };
}
