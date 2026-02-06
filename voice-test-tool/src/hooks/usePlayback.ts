import { useCallback, useEffect, useRef } from "react";
import { usePlaybackStore } from "../stores/playbackStore";
import * as tauri from "../lib/tauri";

/** Polling interval for playback position (ms) */
const POSITION_POLL_INTERVAL = 100;

/**
 * Hook for playback control.
 * Combines playbackStore state with Tauri command invocations.
 *
 * When audio is playing, polls `getPlaybackState()` every 100ms to keep
 * the frontend position in sync with the backend's real-time status.
 */
export function usePlayback() {
  const store = usePlaybackStore();
  const pollRef = useRef<ReturnType<typeof setInterval> | null>(null);

  // -- Polling helpers -------------------------------------------------------

  /** Start polling the backend for playback position */
  const startPolling = useCallback(() => {
    // Avoid duplicate intervals
    if (pollRef.current !== null) return;

    pollRef.current = setInterval(async () => {
      try {
        const state = await tauri.getPlaybackState();
        // Merge backend state while preserving frontend-only fields
        const currentPlayback = usePlaybackStore.getState().playback;
        usePlaybackStore.getState().setPlayback({
          ...state,
          loop_enabled: currentPlayback.loop_enabled,
        });

        // Auto-stop polling when backend reports not playing and not paused
        if (!state.is_playing && !state.is_paused) {
          stopPolling();
        }
      } catch (err) {
        console.log("[usePlayback] polling error:", err);
        // Don't crash on transient errors; just keep trying
      }
    }, POSITION_POLL_INTERVAL);
  }, []);

  const stopPolling = useCallback(() => {
    if (pollRef.current !== null) {
      clearInterval(pollRef.current);
      pollRef.current = null;
    }
  }, []);

  // Cleanup on unmount
  useEffect(() => {
    return () => stopPolling();
  }, [stopPolling]);

  // -- Playback commands -----------------------------------------------------

  const playFile = useCallback(
    async (fileId: string) => {
      store.setLoading(true);
      store.setError(null);
      try {
        await tauri.play(fileId);
        store.setCurrentFileId(fileId);
        store.setIsPlaying(true);
        startPolling();
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        store.setError(message);
        throw err;
      } finally {
        store.setLoading(false);
      }
    },
    [store, startPolling]
  );

  const pausePlayback = useCallback(async () => {
    try {
      await tauri.pause();
      store.setIsPaused(true);
      stopPolling();
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      store.setError(message);
      throw err;
    }
  }, [store, stopPolling]);

  const stopPlayback = useCallback(async () => {
    try {
      await tauri.stop();
      stopPolling();
      store.reset();
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      store.setError(message);
      throw err;
    }
  }, [store, stopPolling]);

  const seekTo = useCallback(
    async (positionSec: number) => {
      try {
        await tauri.seek(positionSec);
        store.setPosition(positionSec);
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        store.setError(message);
        throw err;
      }
    },
    [store]
  );

  const changeSpeed = useCallback(
    async (speed: number) => {
      try {
        await tauri.setPlaybackSpeed(speed);
        store.setSpeed(speed);
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        store.setError(message);
        throw err;
      }
    },
    [store]
  );

  const changeVolume = useCallback(
    async (volume: number) => {
      try {
        await tauri.setVolume(volume);
        store.setVolume(volume);
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        store.setError(message);
        throw err;
      }
    },
    [store]
  );

  const refreshState = useCallback(async () => {
    try {
      const state = await tauri.getPlaybackState();
      const currentPlayback = usePlaybackStore.getState().playback;
      store.setPlayback({
        ...state,
        loop_enabled: currentPlayback.loop_enabled,
      });
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      store.setError(message);
    }
  }, [store]);

  return {
    playback: store.playback,
    isLoading: store.isLoading,
    error: store.error,
    playFile,
    pausePlayback,
    stopPlayback,
    seekTo,
    changeSpeed,
    changeVolume,
    toggleLoop: store.toggleLoop,
    refreshState,
    startPolling,
    stopPolling,
  };
}
