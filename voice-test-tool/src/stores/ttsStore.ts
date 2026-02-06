import { create } from "zustand";
import { immer } from "zustand/middleware/immer";
import type { TtsTestCase, TtsTestSession, TtsTestStatus } from "../lib/tauri";

export interface TtsState {
  /** List of loaded test cases from CSV */
  testCases: TtsTestCase[];
  /** Path to the selected CSV file */
  selectedCsvPath: string | null;
  /** Currently selected test case ID for preview */
  selectedTestCaseId: string | null;
  /** Whether TTS operations are in progress */
  isLoading: boolean;
  /** Error message if any operation failed */
  error: string | null;
  /** ID of the test case currently being previewed */
  previewingId: string | null;
  /** Current test execution session */
  currentSession: TtsTestSession | null;
}

export interface TtsActions {
  /** Set the list of test cases */
  setTestCases: (cases: TtsTestCase[]) => void;
  /** Set the CSV file path */
  setCsvPath: (path: string | null) => void;
  /** Set the selected test case ID */
  setSelectedTestCaseId: (id: string | null) => void;
  /** Set loading state */
  setLoading: (loading: boolean) => void;
  /** Set error message */
  setError: (error: string | null) => void;
  /** Set the ID of the test case being previewed */
  setPreviewingId: (id: string | null) => void;
  /** Set the current test session */
  setCurrentSession: (session: TtsTestSession | null) => void;
  /** Update the status of the current session */
  updateSessionStatus: (status: TtsTestStatus) => void;
  /** Reset the store to initial state */
  reset: () => void;
}

export type TtsStore = TtsState & TtsActions;

export const useTtsStore = create<TtsStore>()(
  immer((set) => ({
    testCases: [],
    selectedCsvPath: null,
    selectedTestCaseId: null,
    isLoading: false,
    error: null,
    previewingId: null,
    currentSession: null,

    setTestCases: (cases) =>
      set((state) => {
        state.testCases = cases;
      }),

    setCsvPath: (path) =>
      set((state) => {
        state.selectedCsvPath = path;
      }),

    setSelectedTestCaseId: (id) =>
      set((state) => {
        state.selectedTestCaseId = id;
      }),

    setLoading: (loading) =>
      set((state) => {
        state.isLoading = loading;
      }),

    setError: (error) =>
      set((state) => {
        state.error = error;
      }),

    setPreviewingId: (id) =>
      set((state) => {
        state.previewingId = id;
      }),

    setCurrentSession: (session) =>
      set((state) => {
        state.currentSession = session;
      }),

    updateSessionStatus: (status) =>
      set((state) => {
        if (state.currentSession) {
          state.currentSession.status = status;
        }
      }),

    reset: () =>
      set((state) => {
        state.testCases = [];
        state.selectedCsvPath = null;
        state.selectedTestCaseId = null;
        state.isLoading = false;
        state.error = null;
        state.previewingId = null;
        state.currentSession = null;
      }),
  }))
);
