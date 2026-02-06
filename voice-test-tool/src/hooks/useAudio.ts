import { useCallback } from "react";
import { useAudioStore } from "../stores/audioStore";
import * as tauri from "../lib/tauri";

/**
 * Hook for audio file management.
 * Combines audioStore state with Tauri command invocations.
 */
export function useAudio() {
  const store = useAudioStore();

  const loadFile = useCallback(
    async (path: string) => {
      store.setLoading(true);
      store.setError(null);
      try {
        const file = await tauri.loadAudioFile(path);
        store.addFile(file);
        return file;
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

  const refreshFiles = useCallback(async () => {
    store.setLoading(true);
    store.setError(null);
    try {
      const files = await tauri.listAudioFiles();
      store.setFiles(files);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      store.setError(message);
    } finally {
      store.setLoading(false);
    }
  }, [store]);

  const removeFile = useCallback(
    async (fileId: string) => {
      try {
        await tauri.removeAudioFile(fileId);
        store.removeFile(fileId);
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        store.setError(message);
        throw err;
      }
    },
    [store]
  );

  const getWaveform = useCallback(
    async (fileId: string, resolution: number = 1000) => {
      try {
        return await tauri.getWaveformData(fileId, resolution);
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        store.setError(message);
        throw err;
      }
    },
    [store]
  );

  return {
    files: store.files,
    selectedFileId: store.selectedFileId,
    isLoading: store.isLoading,
    error: store.error,
    setSelectedFileId: store.setSelectedFileId,
    loadFile,
    refreshFiles,
    removeFile,
    getWaveform,
    clearFiles: store.clearFiles,
  };
}
