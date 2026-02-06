import { create } from "zustand";
import { immer } from "zustand/middleware/immer";

export type Theme = "light" | "dark" | "system";
export type Language = "ja" | "en";
export type SidebarTab = "files" | "playlists" | "scenarios" | "ttsTest";

export interface UiState {
  /** Current theme setting */
  theme: Theme;
  /** Whether the sidebar is visible */
  sidebarVisible: boolean;
  /** Active sidebar tab */
  activeSidebarTab: SidebarTab;
  /** Current UI language */
  language: Language;
  /** Whether the settings modal is open */
  settingsOpen: boolean;
  /** Whether the bottom panel (log viewer) is visible */
  bottomPanelVisible: boolean;
  /** Height of the bottom panel in pixels */
  bottomPanelHeight: number;
}

export interface UiActions {
  /** Set the theme */
  setTheme: (theme: Theme) => void;
  /** Toggle sidebar visibility */
  toggleSidebar: () => void;
  /** Set sidebar visibility */
  setSidebarVisible: (visible: boolean) => void;
  /** Set active sidebar tab */
  setActiveSidebarTab: (tab: SidebarTab) => void;
  /** Set the UI language */
  setLanguage: (language: Language) => void;
  /** Toggle settings modal */
  toggleSettings: () => void;
  /** Set settings modal open state */
  setSettingsOpen: (open: boolean) => void;
  /** Toggle bottom panel visibility */
  toggleBottomPanel: () => void;
  /** Set bottom panel height */
  setBottomPanelHeight: (height: number) => void;
}

export type UiStore = UiState & UiActions;

export const useUiStore = create<UiStore>()(
  immer((set) => ({
    theme: "system",
    sidebarVisible: true,
    activeSidebarTab: "files",
    language: "ja",
    settingsOpen: false,
    bottomPanelVisible: true,
    bottomPanelHeight: 200,

    setTheme: (theme) =>
      set((state) => {
        state.theme = theme;
      }),

    toggleSidebar: () =>
      set((state) => {
        state.sidebarVisible = !state.sidebarVisible;
      }),

    setSidebarVisible: (visible) =>
      set((state) => {
        state.sidebarVisible = visible;
      }),

    setActiveSidebarTab: (tab) =>
      set((state) => {
        state.activeSidebarTab = tab;
      }),

    setLanguage: (language) =>
      set((state) => {
        state.language = language;
      }),

    toggleSettings: () =>
      set((state) => {
        state.settingsOpen = !state.settingsOpen;
      }),

    setSettingsOpen: (open) =>
      set((state) => {
        state.settingsOpen = open;
      }),

    toggleBottomPanel: () =>
      set((state) => {
        state.bottomPanelVisible = !state.bottomPanelVisible;
      }),

    setBottomPanelHeight: (height) =>
      set((state) => {
        state.bottomPanelHeight = Math.max(100, Math.min(500, height));
      }),
  }))
);
