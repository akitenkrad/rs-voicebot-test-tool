import { useCallback, useEffect, useRef } from "react";
import { useDeviceStore } from "../stores/deviceStore";
import * as tauri from "../lib/tauri";

/** Polling interval for device status when a device is active (ms) */
const DEVICE_STATUS_POLL_INTERVAL = 5000;

/**
 * Hook for virtual device management.
 * Combines deviceStore state with Tauri command invocations.
 *
 * When an active device exists, polls `getDeviceStatus()` every 5s
 * to keep the frontend in sync with the backend's real-time status.
 */
export function useDevice() {
  const store = useDeviceStore();
  const pollRef = useRef<ReturnType<typeof setInterval> | null>(null);

  // -- Polling helpers -------------------------------------------------------

  const startStatusPolling = useCallback(() => {
    if (pollRef.current !== null) return;

    pollRef.current = setInterval(async () => {
      try {
        const status = await tauri.getDeviceStatus();
        useDeviceStore.getState().setStatus(status);
      } catch (err) {
        console.log("[useDevice] status polling error:", err);
      }
    }, DEVICE_STATUS_POLL_INTERVAL);
  }, []);

  const stopStatusPolling = useCallback(() => {
    if (pollRef.current !== null) {
      clearInterval(pollRef.current);
      pollRef.current = null;
    }
  }, []);

  // Cleanup on unmount
  useEffect(() => {
    return () => stopStatusPolling();
  }, [stopStatusPolling]);

  // -- Device commands -------------------------------------------------------

  const createDevice = useCallback(
    async (name: string) => {
      store.setLoading(true);
      store.setError(null);
      try {
        const device = await tauri.createVirtualDevice(name);
        store.setActiveDeviceId(device.id);
        await refreshDevices();
        await refreshStatus();
        startStatusPolling();
        return device;
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        store.setError(message);
        throw err;
      } finally {
        store.setLoading(false);
      }
    },
    [store, startStatusPolling]
  );

  const destroyDevice = useCallback(async () => {
    store.setLoading(true);
    store.setError(null);
    try {
      await tauri.destroyVirtualDevice();
      store.setActiveDeviceId(null);
      store.setStatus(null);
      stopStatusPolling();
      await refreshDevices();
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      store.setError(message);
      throw err;
    } finally {
      store.setLoading(false);
    }
  }, [store, stopStatusPolling]);

  const refreshDevices = useCallback(async () => {
    try {
      const devices = await tauri.listAudioDevices();
      useDeviceStore.getState().setDevices(devices);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      useDeviceStore.getState().setError(message);
    }
  }, []);

  const setAsDefault = useCallback(
    async (deviceId: string) => {
      store.setLoading(true);
      try {
        await tauri.setDefaultInputDevice(deviceId);
        await refreshStatus();
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

  const refreshStatus = useCallback(async () => {
    try {
      const status = await tauri.getDeviceStatus();
      useDeviceStore.getState().setStatus(status);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      useDeviceStore.getState().setError(message);
    }
  }, []);

  return {
    devices: store.devices,
    activeDeviceId: store.activeDeviceId,
    status: store.status,
    isLoading: store.isLoading,
    error: store.error,
    createDevice,
    destroyDevice,
    refreshDevices,
    setAsDefault,
    refreshStatus,
    startStatusPolling,
    stopStatusPolling,
  };
}
