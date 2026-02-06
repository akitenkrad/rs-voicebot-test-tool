import { create } from "zustand";
import { immer } from "zustand/middleware/immer";
import type { AudioFileInfo } from "../types/audio";

export interface AudioState {
  /** List of loaded audio files */
  files: AudioFileInfo[];
  /** Currently selected file ID */
  selectedFileId: string | null;
  /** Whether files are currently being loaded */
  isLoading: boolean;
  /** Error message if loading failed */
  error: string | null;
}

export interface AudioActions {
  /** Set the list of audio files */
  setFiles: (files: AudioFileInfo[]) => void;
  /** Add a single audio file to the list */
  addFile: (file: AudioFileInfo) => void;
  /** Remove a file by ID */
  removeFile: (fileId: string) => void;
  /** Set the currently selected file */
  setSelectedFileId: (fileId: string | null) => void;
  /** Set loading state */
  setLoading: (isLoading: boolean) => void;
  /** Set error message */
  setError: (error: string | null) => void;
  /** Clear all files */
  clearFiles: () => void;
}

export type AudioStore = AudioState & AudioActions;

export const useAudioStore = create<AudioStore>()(
  immer((set) => ({
    files: [],
    selectedFileId: null,
    isLoading: false,
    error: null,

    setFiles: (files) =>
      set((state) => {
        state.files = files;
      }),

    addFile: (file) =>
      set((state) => {
        state.files.push(file);
      }),

    removeFile: (fileId) =>
      set((state) => {
        state.files = state.files.filter((f) => f.id !== fileId);
        if (state.selectedFileId === fileId) {
          state.selectedFileId = null;
        }
      }),

    setSelectedFileId: (fileId) =>
      set((state) => {
        state.selectedFileId = fileId;
      }),

    setLoading: (isLoading) =>
      set((state) => {
        state.isLoading = isLoading;
      }),

    setError: (error) =>
      set((state) => {
        state.error = error;
      }),

    clearFiles: () =>
      set((state) => {
        state.files = [];
        state.selectedFileId = null;
        state.error = null;
      }),
  }))
);
