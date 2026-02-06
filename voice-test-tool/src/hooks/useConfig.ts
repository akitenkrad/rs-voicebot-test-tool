import { useState, useCallback } from "react";
import * as tauri from "../lib/tauri";
import type { AppConfig } from "../lib/tauri";

/**
 * Hook for application configuration management.
 * Wraps Tauri config commands with local loading/error state.
 */
export function useConfig() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchConfig = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      const result = await tauri.getConfig();
      setConfig(result);
      return result;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      throw err;
    } finally {
      setIsLoading(false);
    }
  }, []);

  const saveConfig = useCallback(async (newConfig: AppConfig) => {
    setIsLoading(true);
    setError(null);
    try {
      await tauri.updateConfig(newConfig);
      setConfig(newConfig);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      throw err;
    } finally {
      setIsLoading(false);
    }
  }, []);

  return {
    config,
    isLoading,
    error,
    fetchConfig,
    saveConfig,
  };
}
