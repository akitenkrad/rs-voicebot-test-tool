import { useEffect } from "react";

export interface KeyboardShortcutCallbacks {
  onPlayPause?: () => void;
  onStop?: () => void;
  onSeekForward?: () => void;
  onSeekBackward?: () => void;
  onVolumeUp?: () => void;
  onVolumeDown?: () => void;
  onNextFile?: () => void;
  onPrevFile?: () => void;
  onOpenFile?: () => void;
  onOpenSettings?: () => void;
  onRunScenario?: () => void;
  onStopScenario?: () => void;
}

/**
 * Detect if the current platform is macOS.
 * Uses navigator.platform (deprecated but widely supported) with
 * navigator.userAgent as fallback.
 */
function isMacOS(): boolean {
  // navigator.platform is deprecated but still the most reliable check
  if (typeof navigator !== "undefined") {
    if (navigator.platform) {
      return navigator.platform.toUpperCase().includes("MAC");
    }
    return /macintosh|mac os x/i.test(navigator.userAgent);
  }
  return false;
}

/**
 * Hook that registers global keyboard shortcuts for the application.
 *
 * Shortcuts:
 * | Action           | Windows/Linux  | macOS       |
 * |------------------|----------------|-------------|
 * | Play/Pause       | Space          | Space       |
 * | Stop             | Escape         | Escape      |
 * | Next file        | Ctrl+Right     | Cmd+Right   |
 * | Previous file    | Ctrl+Left      | Cmd+Left    |
 * | Seek forward 5s  | Right          | Right       |
 * | Seek backward 5s | Left           | Left        |
 * | Volume up        | Up             | Up          |
 * | Volume down      | Down           | Down        |
 * | Open file        | Ctrl+O         | Cmd+O       |
 * | Settings         | Ctrl+,         | Cmd+,       |
 * | Run scenario     | F5             | F5          |
 * | Stop scenario    | Shift+F5       | Shift+F5    |
 */
export function useKeyboardShortcuts(callbacks: KeyboardShortcutCallbacks): void {
  useEffect(() => {
    const mac = isMacOS();

    /**
     * Returns true if the platform-appropriate modifier key is pressed.
     * On macOS this is the Meta (Cmd) key; on Windows/Linux it is Ctrl.
     */
    const hasModifier = (e: KeyboardEvent): boolean => {
      return mac ? e.metaKey : e.ctrlKey;
    };

    /**
     * Returns true if no modifier key (Ctrl, Meta, Alt, Shift) is pressed.
     */
    const noModifiers = (e: KeyboardEvent): boolean => {
      return !e.ctrlKey && !e.metaKey && !e.altKey && !e.shiftKey;
    };

    const handleKeyDown = (e: KeyboardEvent) => {
      // Skip if the user is focused on an input element (text inputs, textareas, selects)
      const target = e.target as HTMLElement;
      const tagName = target.tagName.toLowerCase();
      if (
        tagName === "input" ||
        tagName === "textarea" ||
        tagName === "select" ||
        target.isContentEditable
      ) {
        return;
      }

      // Space - Play/Pause
      if (e.code === "Space" && noModifiers(e)) {
        e.preventDefault(); // Prevent page scroll
        callbacks.onPlayPause?.();
        return;
      }

      // Escape - Stop
      if (e.code === "Escape" && noModifiers(e)) {
        e.preventDefault();
        callbacks.onStop?.();
        return;
      }

      // Ctrl/Cmd + ArrowRight - Next file
      if (e.code === "ArrowRight" && hasModifier(e) && !e.shiftKey && !e.altKey) {
        e.preventDefault();
        callbacks.onNextFile?.();
        return;
      }

      // Ctrl/Cmd + ArrowLeft - Previous file
      if (e.code === "ArrowLeft" && hasModifier(e) && !e.shiftKey && !e.altKey) {
        e.preventDefault();
        callbacks.onPrevFile?.();
        return;
      }

      // ArrowRight (no modifiers) - Seek forward 5s
      if (e.code === "ArrowRight" && noModifiers(e)) {
        e.preventDefault();
        callbacks.onSeekForward?.();
        return;
      }

      // ArrowLeft (no modifiers) - Seek backward 5s
      if (e.code === "ArrowLeft" && noModifiers(e)) {
        e.preventDefault();
        callbacks.onSeekBackward?.();
        return;
      }

      // ArrowUp (no modifiers) - Volume up
      if (e.code === "ArrowUp" && noModifiers(e)) {
        e.preventDefault();
        callbacks.onVolumeUp?.();
        return;
      }

      // ArrowDown (no modifiers) - Volume down
      if (e.code === "ArrowDown" && noModifiers(e)) {
        e.preventDefault();
        callbacks.onVolumeDown?.();
        return;
      }

      // Ctrl/Cmd + O - Open file
      if (e.code === "KeyO" && hasModifier(e) && !e.shiftKey && !e.altKey) {
        e.preventDefault();
        callbacks.onOpenFile?.();
        return;
      }

      // Ctrl/Cmd + , - Settings
      if (e.code === "Comma" && hasModifier(e) && !e.shiftKey && !e.altKey) {
        e.preventDefault();
        callbacks.onOpenSettings?.();
        return;
      }

      // F5 (no modifiers) - Run scenario
      if (e.code === "F5" && !e.ctrlKey && !e.metaKey && !e.altKey && !e.shiftKey) {
        e.preventDefault();
        callbacks.onRunScenario?.();
        return;
      }

      // Shift + F5 - Stop scenario
      if (e.code === "F5" && e.shiftKey && !e.ctrlKey && !e.metaKey && !e.altKey) {
        e.preventDefault();
        callbacks.onStopScenario?.();
        return;
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [callbacks]);
}
