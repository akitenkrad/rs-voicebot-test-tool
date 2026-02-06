import { useCallback } from "react";
import { usePlaylistStore } from "../stores/playlistStore";
import { useAudioStore } from "../stores/audioStore";
import * as tauri from "../lib/tauri";

/**
 * Hook for playlist management.
 * Combines playlistStore state with Tauri command invocations.
 */
export function usePlaylist() {
  const store = usePlaylistStore();
  const selectedFileId = useAudioStore((s) => s.selectedFileId);

  const fetchPlaylists = useCallback(async () => {
    store.setLoading(true);
    store.setError(null);
    try {
      const playlists = await tauri.listPlaylists();
      store.setPlaylists(playlists);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      store.setError(message);
    } finally {
      store.setLoading(false);
    }
  }, [store]);

  const createNewPlaylist = useCallback(
    async (name: string) => {
      store.setLoading(true);
      store.setError(null);
      try {
        const playlist = await tauri.createPlaylist(name);
        store.addPlaylist(playlist);
        store.setSelectedPlaylistId(playlist.id);
        return playlist;
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

  const deletePlaylist = useCallback(
    async (playlistId: string) => {
      store.setError(null);
      try {
        await tauri.deletePlaylist(playlistId);
        store.removePlaylist(playlistId);
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        store.setError(message);
        throw err;
      }
    },
    [store]
  );

  const refreshSelectedPlaylist = useCallback(async () => {
    const id = usePlaylistStore.getState().selectedPlaylistId;
    if (!id) return;
    try {
      const playlist = await tauri.getPlaylist(id);
      store.updatePlaylist(playlist);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      store.setError(message);
    }
  }, [store]);

  const addFileToPlaylist = useCallback(
    async (preSilenceSec = 0, postSilenceSec = 0) => {
      const playlistId = usePlaylistStore.getState().selectedPlaylistId;
      const fileId = useAudioStore.getState().selectedFileId;
      if (!playlistId || !fileId) return;
      store.setError(null);
      try {
        await tauri.addToPlaylist(playlistId, fileId, preSilenceSec, postSilenceSec);
        await refreshSelectedPlaylist();
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        store.setError(message);
        throw err;
      }
    },
    [store, refreshSelectedPlaylist]
  );

  const removeFileFromPlaylist = useCallback(
    async (itemId: string) => {
      const playlistId = usePlaylistStore.getState().selectedPlaylistId;
      if (!playlistId) return;
      store.setError(null);
      try {
        await tauri.removeFromPlaylist(playlistId, itemId);
        await refreshSelectedPlaylist();
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        store.setError(message);
        throw err;
      }
    },
    [store, refreshSelectedPlaylist]
  );

  const reorderItems = useCallback(
    async (itemIds: string[]) => {
      const playlistId = usePlaylistStore.getState().selectedPlaylistId;
      if (!playlistId) return;
      store.setError(null);
      try {
        await tauri.reorderPlaylist(playlistId, itemIds);
        await refreshSelectedPlaylist();
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        store.setError(message);
        throw err;
      }
    },
    [store, refreshSelectedPlaylist]
  );

  const updateSilence = useCallback(
    async (itemId: string, preSilenceSec: number, postSilenceSec: number) => {
      const playlistId = usePlaylistStore.getState().selectedPlaylistId;
      if (!playlistId) return;
      store.setError(null);
      try {
        await tauri.updatePlaylistItemSilence(
          playlistId,
          itemId,
          preSilenceSec,
          postSilenceSec
        );
        await refreshSelectedPlaylist();
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        store.setError(message);
        throw err;
      }
    },
    [store, refreshSelectedPlaylist]
  );

  const playSelectedPlaylist = useCallback(
    async (loopMode: boolean) => {
      const playlistId = usePlaylistStore.getState().selectedPlaylistId;
      if (!playlistId) return;
      store.setError(null);
      try {
        await tauri.playPlaylist(playlistId, loopMode);
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        store.setError(message);
        throw err;
      }
    },
    [store]
  );

  const stopPlaylistPlayback = useCallback(async () => {
    store.setError(null);
    try {
      await tauri.stopPlaylist();
      store.setPlaybackStatus(null);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      store.setError(message);
      throw err;
    }
  }, [store]);

  const refreshPlaybackStatus = useCallback(async () => {
    try {
      const status = await tauri.getPlaylistPlaybackStatus();
      store.setPlaybackStatus(status);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      store.setError(message);
    }
  }, [store]);

  // Derive the selected playlist from the store
  const selectedPlaylist =
    store.playlists.find((p) => p.id === store.selectedPlaylistId) ?? null;

  return {
    // State
    playlists: store.playlists,
    selectedPlaylist,
    selectedPlaylistId: store.selectedPlaylistId,
    playbackStatus: store.playbackStatus,
    isLoading: store.isLoading,
    error: store.error,
    selectedFileId,
    // Actions
    fetchPlaylists,
    createNewPlaylist,
    deletePlaylist,
    addFileToPlaylist,
    removeFileFromPlaylist,
    reorderItems,
    updateSilence,
    playSelectedPlaylist,
    stopPlaylistPlayback,
    refreshPlaybackStatus,
    setSelectedPlaylistId: store.setSelectedPlaylistId,
  };
}
