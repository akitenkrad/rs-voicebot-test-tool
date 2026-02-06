import { create } from "zustand";
import { immer } from "zustand/middleware/immer";
import type { PlaybackState } from "../types/audio";

export interface PlaybackStoreState {
  /** Current playback state */
  playback: PlaybackState;
  /** Whether a playback operation is in progress */
  isLoading: boolean;
  /** Error message if playback operation failed */
  error: string | null;
}

export interface PlaybackActions {
  /** Update the full playback state */
  setPlayback: (playback: PlaybackState) => void;
  /** Update playback position */
  setPosition: (positionSec: number) => void;
  /** Set playing status */
  setIsPlaying: (isPlaying: boolean) => void;
  /** Set paused status */
  setIsPaused: (isPaused: boolean) => void;
  /** Set volume (0.0 to 1.0) */
  setVolume: (volume: number) => void;
  /** Set playback speed (0.5 to 2.0) */
  setSpeed: (speed: number) => void;
  /** Toggle loop mode */
  toggleLoop: () => void;
  /** Set the currently playing file ID */
  setCurrentFileId: (fileId: string | null) => void;
  /** Set loading state */
  setLoading: (isLoading: boolean) => void;
  /** Set error message */
  setError: (error: string | null) => void;
  /** Reset playback state to defaults */
  reset: () => void;
}

export type PlaybackStore = PlaybackStoreState & PlaybackActions;

const defaultPlayback: PlaybackState = {
  file_id: null,
  is_playing: false,
  is_paused: false,
  current_position_sec: 0,
  total_duration_sec: 0,
  volume: 0.8,
  speed: 1.0,
  loop_enabled: false,
};

export const usePlaybackStore = create<PlaybackStore>()(
  immer((set) => ({
    playback: { ...defaultPlayback },
    isLoading: false,
    error: null,

    setPlayback: (playback) =>
      set((state) => {
        state.playback = playback;
      }),

    setPosition: (positionSec) =>
      set((state) => {
        state.playback.current_position_sec = positionSec;
      }),

    setIsPlaying: (isPlaying) =>
      set((state) => {
        state.playback.is_playing = isPlaying;
        if (isPlaying) {
          state.playback.is_paused = false;
        }
      }),

    setIsPaused: (isPaused) =>
      set((state) => {
        state.playback.is_paused = isPaused;
        if (isPaused) {
          state.playback.is_playing = false;
        }
      }),

    setVolume: (volume) =>
      set((state) => {
        state.playback.volume = Math.max(0, Math.min(1, volume));
      }),

    setSpeed: (speed) =>
      set((state) => {
        state.playback.speed = Math.max(0.5, Math.min(2.0, speed));
      }),

    toggleLoop: () =>
      set((state) => {
        state.playback.loop_enabled = !state.playback.loop_enabled;
      }),

    setCurrentFileId: (fileId) =>
      set((state) => {
        state.playback.file_id = fileId;
      }),

    setLoading: (isLoading) =>
      set((state) => {
        state.isLoading = isLoading;
      }),

    setError: (error) =>
      set((state) => {
        state.error = error;
      }),

    reset: () =>
      set((state) => {
        state.playback = { ...defaultPlayback };
        state.error = null;
      }),
  }))
);
