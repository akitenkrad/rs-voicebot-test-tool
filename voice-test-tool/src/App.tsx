import { useEffect, useCallback } from "react";
import { useUiStore } from "./stores/uiStore";
import { usePlaybackStore } from "./stores/playbackStore";
import { useDevice } from "./hooks/useDevice";
import { useKeyboardShortcuts } from "./hooks/useKeyboardShortcuts";
import * as tauri from "./lib/tauri";
import { TooltipProvider } from "./components/ui/tooltip";
import { Header } from "./components/Header/Header";
import { Sidebar } from "./components/Sidebar/Sidebar";
import { MainContent } from "./components/MainContent/MainContent";
import { BottomPanel } from "./components/BottomPanel/BottomPanel";
import { SettingsModal } from "./components/Modals/SettingsModal";

function App() {
  const theme = useUiStore((state) => state.theme);
  const toggleSettings = useUiStore((state) => state.toggleSettings);
  const { refreshDevices, refreshStatus } = useDevice();

  // Apply theme class to the document root
  useEffect(() => {
    const root = document.documentElement;

    if (theme === "system") {
      const prefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
      root.classList.toggle("dark", prefersDark);

      const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
      const handler = (e: MediaQueryListEvent) => {
        root.classList.toggle("dark", e.matches);
      };
      mediaQuery.addEventListener("change", handler);
      return () => mediaQuery.removeEventListener("change", handler);
    } else {
      root.classList.toggle("dark", theme === "dark");
    }
  }, [theme]);

  // Initialize: populate device list and detect existing virtual device on mount
  useEffect(() => {
    const init = async () => {
      try {
        await refreshDevices();
        console.log("[App] device list loaded");
      } catch (err) {
        console.log("[App] failed to load device list:", err);
      }

      try {
        await refreshStatus();
        console.log("[App] device status loaded");
      } catch (err) {
        console.log("[App] failed to load device status:", err);
      }
    };

    init();
    // Intentionally run only on mount
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Keyboard shortcut callbacks
  const handlePlayPause = useCallback(async () => {
    const playback = usePlaybackStore.getState().playback;
    if (playback.is_playing) {
      try {
        await tauri.pause();
        usePlaybackStore.getState().setIsPaused(true);
      } catch (err) {
        console.error("[App] pause failed:", err);
      }
    } else if (playback.file_id) {
      try {
        await tauri.play(playback.file_id);
        usePlaybackStore.getState().setIsPlaying(true);
      } catch (err) {
        console.error("[App] play failed:", err);
      }
    }
  }, []);

  const handleStop = useCallback(async () => {
    try {
      await tauri.stop();
      usePlaybackStore.getState().setIsPlaying(false);
      usePlaybackStore.getState().setIsPaused(false);
      usePlaybackStore.getState().setPosition(0);
    } catch (err) {
      console.error("[App] stop failed:", err);
    }
  }, []);

  const handleSeekForward = useCallback(async () => {
    const playback = usePlaybackStore.getState().playback;
    const newPos = Math.min(
      playback.current_position_sec + 5,
      playback.total_duration_sec
    );
    try {
      await tauri.seek(newPos);
      usePlaybackStore.getState().setPosition(newPos);
    } catch (err) {
      console.error("[App] seek forward failed:", err);
    }
  }, []);

  const handleSeekBackward = useCallback(async () => {
    const playback = usePlaybackStore.getState().playback;
    const newPos = Math.max(playback.current_position_sec - 5, 0);
    try {
      await tauri.seek(newPos);
      usePlaybackStore.getState().setPosition(newPos);
    } catch (err) {
      console.error("[App] seek backward failed:", err);
    }
  }, []);

  const handleVolumeUp = useCallback(async () => {
    const playback = usePlaybackStore.getState().playback;
    const newVol = Math.min(playback.volume + 0.1, 1.0);
    try {
      await tauri.setVolume(newVol);
      usePlaybackStore.getState().setVolume(newVol);
    } catch (err) {
      console.error("[App] volume up failed:", err);
    }
  }, []);

  const handleVolumeDown = useCallback(async () => {
    const playback = usePlaybackStore.getState().playback;
    const newVol = Math.max(playback.volume - 0.1, 0.0);
    try {
      await tauri.setVolume(newVol);
      usePlaybackStore.getState().setVolume(newVol);
    } catch (err) {
      console.error("[App] volume down failed:", err);
    }
  }, []);

  const handleOpenFile = useCallback(() => {
    // Placeholder: file dialog will be implemented in a future phase
    console.log("[App] open file shortcut triggered");
  }, []);

  const handleOpenSettings = useCallback(() => {
    toggleSettings();
  }, [toggleSettings]);

  const handleRunScenario = useCallback(() => {
    // Placeholder: scenario execution will be wired in a future phase
    console.log("[App] run scenario shortcut triggered");
  }, []);

  const handleStopScenario = useCallback(() => {
    // Placeholder: scenario stop will be wired in a future phase
    console.log("[App] stop scenario shortcut triggered");
  }, []);

  // Register keyboard shortcuts
  useKeyboardShortcuts({
    onPlayPause: handlePlayPause,
    onStop: handleStop,
    onSeekForward: handleSeekForward,
    onSeekBackward: handleSeekBackward,
    onVolumeUp: handleVolumeUp,
    onVolumeDown: handleVolumeDown,
    onNextFile: handleOpenFile, // Reuse placeholder for now
    onPrevFile: handleOpenFile, // Reuse placeholder for now
    onOpenFile: handleOpenFile,
    onOpenSettings: handleOpenSettings,
    onRunScenario: handleRunScenario,
    onStopScenario: handleStopScenario,
  });

  return (
    <TooltipProvider delayDuration={300}>
      <div className="flex h-screen w-screen flex-col overflow-hidden bg-background text-foreground">
        {/* Header - full width top bar */}
        <Header />

        {/* Middle area: Sidebar + MainContent */}
        <div className="flex flex-1 overflow-hidden">
          <Sidebar />
          <MainContent />
        </div>

        {/* Bottom panel - log viewer */}
        <BottomPanel />

        {/* Modals */}
        <SettingsModal />
      </div>
    </TooltipProvider>
  );
}

export default App;
