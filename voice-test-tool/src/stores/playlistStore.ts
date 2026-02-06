import { create } from "zustand";
import { immer } from "zustand/middleware/immer";
import type { PlaylistInfo, PlaylistPlaybackInfo } from "../types/audio";

export interface PlaylistState {
  /** All available playlists */
  playlists: PlaylistInfo[];
  /** Currently selected playlist ID */
  selectedPlaylistId: string | null;
  /** Current playlist playback status */
  playbackStatus: PlaylistPlaybackInfo | null;
  /** Whether a playlist operation is in progress */
  isLoading: boolean;
  /** Error message if an operation failed */
  error: string | null;
}

export interface PlaylistActions {
  /** Set the full list of playlists */
  setPlaylists: (playlists: PlaylistInfo[]) => void;
  /** Add a single playlist to the list */
  addPlaylist: (playlist: PlaylistInfo) => void;
  /** Remove a playlist by ID */
  removePlaylist: (id: string) => void;
  /** Set the currently selected playlist ID */
  setSelectedPlaylistId: (id: string | null) => void;
  /** Set the playlist playback status */
  setPlaybackStatus: (status: PlaylistPlaybackInfo | null) => void;
  /** Update an existing playlist (replace by ID) */
  updatePlaylist: (playlist: PlaylistInfo) => void;
  /** Set loading state */
  setLoading: (loading: boolean) => void;
  /** Set error message */
  setError: (error: string | null) => void;
}

export type PlaylistStore = PlaylistState & PlaylistActions;

export const usePlaylistStore = create<PlaylistStore>()(
  immer((set) => ({
    playlists: [],
    selectedPlaylistId: null,
    playbackStatus: null,
    isLoading: false,
    error: null,

    setPlaylists: (playlists) =>
      set((state) => {
        state.playlists = playlists;
      }),

    addPlaylist: (playlist) =>
      set((state) => {
        state.playlists.push(playlist);
      }),

    removePlaylist: (id) =>
      set((state) => {
        state.playlists = state.playlists.filter((p) => p.id !== id);
        if (state.selectedPlaylistId === id) {
          state.selectedPlaylistId = null;
        }
      }),

    setSelectedPlaylistId: (id) =>
      set((state) => {
        state.selectedPlaylistId = id;
      }),

    setPlaybackStatus: (status) =>
      set((state) => {
        state.playbackStatus = status;
      }),

    updatePlaylist: (playlist) =>
      set((state) => {
        const index = state.playlists.findIndex((p) => p.id === playlist.id);
        if (index !== -1) {
          state.playlists[index] = playlist;
        }
      }),

    setLoading: (loading) =>
      set((state) => {
        state.isLoading = loading;
      }),

    setError: (error) =>
      set((state) => {
        state.error = error;
      }),
  }))
);
