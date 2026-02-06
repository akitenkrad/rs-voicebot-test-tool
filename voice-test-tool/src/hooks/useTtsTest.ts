import { useCallback, useRef, useEffect } from "react";
import { useTtsStore } from "../stores/ttsStore";
import * as tauri from "../lib/tauri";
import type { TtsTestCase, TtsTestSession } from "../lib/tauri";
import { open, save } from "@tauri-apps/plugin-dialog";
import { listen } from "@tauri-apps/api/event";

/**
 * Hook for TTS test case management.
 * Handles CSV loading, TTS preview generation, and test execution.
 */
export function useTtsTest() {
  const store = useTtsStore();
  const audioRef = useRef<HTMLAudioElement | null>(null);

  // Listen for TTS test status updates from the backend.
  // IMPORTANT: empty dependency array — register once, never re-register.
  // Use useTtsStore.getState() inside the callback to avoid stale closures.
  useEffect(() => {
    const unlisten = listen<TtsTestSession>("tts-test-status-update", (event) => {
      console.log("[useTtsTest] Received status update:", event.payload);
      useTtsStore.getState().setCurrentSession(event.payload);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  // Polling fallback: periodically fetch session status while a test is active.
  // Guards against event loss from IPC edge cases.
  const pollingRef = useRef<ReturnType<typeof setInterval> | null>(null);

  useEffect(() => {
    const isActive = () => {
      const session = useTtsStore.getState().currentSession;
      if (!session) return false;
      const t = session.status.type;
      return t === "Running" || t === "Paused";
    };

    const poll = async () => {
      const session = useTtsStore.getState().currentSession;
      if (!session) return;
      try {
        const updated = await tauri.getTtsTestSession(session.id);
        useTtsStore.getState().setCurrentSession(updated);

        // Stop polling once we reach a terminal state
        const t = updated.status.type;
        if (t === "Completed" || t === "Aborted" || t === "Failed") {
          console.log(`[useTtsTest] Polling detected terminal status: ${t}`);
          if (pollingRef.current) {
            clearInterval(pollingRef.current);
            pollingRef.current = null;
          }
        }
      } catch (err) {
        console.error("[useTtsTest] Polling error:", err);
      }
    };

    // Start polling when a session becomes active
    if (isActive() && !pollingRef.current) {
      pollingRef.current = setInterval(poll, 3000);
    }

    // Stop polling when session is no longer active
    if (!isActive() && pollingRef.current) {
      clearInterval(pollingRef.current);
      pollingRef.current = null;
    }

    return () => {
      if (pollingRef.current) {
        clearInterval(pollingRef.current);
        pollingRef.current = null;
      }
    };
  }, [store.currentSession?.status.type]);

  /**
   * Load a CSV file from a given path.
   */
  const loadCsvFromPath = useCallback(async (path: string) => {
    store.setLoading(true);
    store.setError(null);

    try {
      const cases = await tauri.loadTtsCsv(path);
      store.setCsvPath(path);
      store.setTestCases(cases);

      console.log(`[useTtsTest] Loaded ${cases.length} test cases from: ${path}`);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      store.setError(message);
      console.error("[useTtsTest] Error loading CSV:", message);
    } finally {
      store.setLoading(false);
    }
  }, [store]);

  /**
   * Open a file dialog and load a CSV file containing test cases.
   */
  const loadCsv = useCallback(async () => {
    store.setLoading(true);
    store.setError(null);

    try {
      const path = await open({
        multiple: false,
        filters: [{ name: "CSV", extensions: ["csv"] }],
      });

      if (!path || Array.isArray(path)) {
        store.setLoading(false);
        return;
      }

      await loadCsvFromPath(path);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      store.setError(message);
      console.error("[useTtsTest] Error loading CSV:", message);
      store.setLoading(false);
    }
  }, [store, loadCsvFromPath]);

  /**
   * Generate TTS audio for a given text and play it.
   */
  const previewTts = useCallback(
    async (testCase: TtsTestCase): Promise<void> => {
      store.setPreviewingId(testCase.id);
      store.setError(null);

      try {
        const bytes = await tauri.previewTts(testCase.text);

        // Stop any currently playing audio
        if (audioRef.current) {
          audioRef.current.pause();
          audioRef.current = null;
        }

        // Convert bytes to audio and play
        const blob = new Blob([bytes], { type: "audio/wav" });
        const url = URL.createObjectURL(blob);
        const audio = new Audio(url);
        audioRef.current = audio;

        audio.onended = () => {
          URL.revokeObjectURL(url);
          store.setPreviewingId(null);
          audioRef.current = null;
        };

        audio.onerror = () => {
          URL.revokeObjectURL(url);
          store.setPreviewingId(null);
          store.setError("Failed to play audio");
          audioRef.current = null;
        };

        await audio.play();
        console.log(`[useTtsTest] Playing TTS preview for case ${testCase.id}`);
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        store.setError(message);
        store.setPreviewingId(null);
        console.error("[useTtsTest] Error generating TTS:", message);
      }
    },
    [store]
  );

  /**
   * Stop any currently playing preview audio.
   */
  const stopPreview = useCallback(() => {
    if (audioRef.current) {
      audioRef.current.pause();
      audioRef.current = null;
    }
    store.setPreviewingId(null);
  }, [store]);

  /**
   * Start TTS test execution with the currently loaded CSV.
   * Returns the session ID if successful, null otherwise.
   */
  const startTest = useCallback(async (): Promise<string | null> => {
    if (!store.selectedCsvPath) {
      store.setError("No CSV file loaded");
      return null;
    }

    store.setLoading(true);
    store.setError(null);

    try {
      const sessionId = await tauri.startTtsTest(store.selectedCsvPath);
      console.log(`[useTtsTest] Started test session: ${sessionId}`);

      // Fetch the initial session state
      const session = await tauri.getTtsTestSession(sessionId);
      store.setCurrentSession(session);

      return sessionId;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      store.setError(message);
      console.error("[useTtsTest] Error starting test:", message);
      return null;
    } finally {
      store.setLoading(false);
    }
  }, [store]);

  /**
   * Pause the current TTS test execution.
   */
  const pauseTest = useCallback(async (): Promise<void> => {
    const session = store.currentSession;
    if (!session) {
      store.setError("No active test session");
      return;
    }

    try {
      await tauri.pauseTtsTest(session.id);
      console.log(`[useTtsTest] Paused test session: ${session.id}`);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      store.setError(message);
      console.error("[useTtsTest] Error pausing test:", message);
    }
  }, [store]);

  /**
   * Resume the current TTS test execution.
   */
  const resumeTest = useCallback(async (): Promise<void> => {
    const session = store.currentSession;
    if (!session) {
      store.setError("No active test session");
      return;
    }

    try {
      await tauri.resumeTtsTest(session.id);
      console.log(`[useTtsTest] Resumed test session: ${session.id}`);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      store.setError(message);
      console.error("[useTtsTest] Error resuming test:", message);
    }
  }, [store]);

  /**
   * Abort the current TTS test execution.
   */
  const abortTest = useCallback(async (): Promise<void> => {
    const session = store.currentSession;
    if (!session) {
      store.setError("No active test session");
      return;
    }

    try {
      await tauri.abortTtsTest(session.id);
      console.log(`[useTtsTest] Aborted test session: ${session.id}`);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      store.setError(message);
      console.error("[useTtsTest] Error aborting test:", message);
    }
  }, [store]);

  /**
   * Refresh the status of the current test session.
   */
  const refreshStatus = useCallback(async (): Promise<void> => {
    const session = store.currentSession;
    if (!session) {
      return;
    }

    try {
      const updatedSession = await tauri.getTtsTestSession(session.id);
      store.setCurrentSession(updatedSession);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      console.error("[useTtsTest] Error refreshing status:", message);
    }
  }, [store]);

  /**
   * Export the current test session results to a ZIP file.
   * Opens a save dialog and exports the session data.
   * Returns the output path if successful, null otherwise.
   */
  const exportResults = useCallback(async (): Promise<string | null> => {
    const session = store.currentSession;
    if (!session) {
      store.setError("No active session to export");
      return null;
    }

    // Open save dialog
    const path = await save({
      defaultPath: `export_${new Date().toISOString().slice(0, 10)}.zip`,
      filters: [{ name: "ZIP Archive", extensions: ["zip"] }],
    });

    if (!path) {
      return null;
    }

    store.setLoading(true);
    store.setError(null);

    try {
      const result = await tauri.exportTtsTest(session.id, path);
      console.log(`[useTtsTest] Exported session to: ${result}`);
      return result;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      store.setError(message);
      console.error("[useTtsTest] Error exporting session:", message);
      return null;
    } finally {
      store.setLoading(false);
    }
  }, [store]);

  /**
   * Reset the store and stop any playing audio.
   */
  const reset = useCallback(() => {
    stopPreview();
    store.reset();
  }, [store, stopPreview]);

  return {
    testCases: store.testCases,
    selectedCsvPath: store.selectedCsvPath,
    selectedTestCaseId: store.selectedTestCaseId,
    isLoading: store.isLoading,
    error: store.error,
    previewingId: store.previewingId,
    currentSession: store.currentSession,
    setSelectedTestCaseId: store.setSelectedTestCaseId,
    loadCsv,
    loadCsvFromPath,
    previewTts,
    stopPreview,
    startTest,
    pauseTest,
    resumeTest,
    abortTest,
    refreshStatus,
    exportResults,
    reset,
  };
}
