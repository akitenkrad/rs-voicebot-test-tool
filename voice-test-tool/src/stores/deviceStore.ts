import { create } from "zustand";
import { immer } from "zustand/middleware/immer";
import type { DeviceInfo, DeviceStatus } from "../types/device";

export interface DeviceState {
  /** List of available audio devices */
  devices: DeviceInfo[];
  /** Currently active virtual device ID */
  activeDeviceId: string | null;
  /** Current device status */
  status: DeviceStatus | null;
  /** Whether device operations are in progress */
  isLoading: boolean;
  /** Error message if device operation failed */
  error: string | null;
}

export interface DeviceActions {
  /** Set the list of devices */
  setDevices: (devices: DeviceInfo[]) => void;
  /** Set the active device ID */
  setActiveDeviceId: (deviceId: string | null) => void;
  /** Update device status */
  setStatus: (status: DeviceStatus | null) => void;
  /** Set loading state */
  setLoading: (isLoading: boolean) => void;
  /** Set error message */
  setError: (error: string | null) => void;
  /** Update a specific device in the list */
  updateDevice: (deviceId: string, updates: Partial<DeviceInfo>) => void;
}

export type DeviceStore = DeviceState & DeviceActions;

export const useDeviceStore = create<DeviceStore>()(
  immer((set) => ({
    devices: [],
    activeDeviceId: null,
    status: null,
    isLoading: false,
    error: null,

    setDevices: (devices) =>
      set((state) => {
        state.devices = devices;
      }),

    setActiveDeviceId: (deviceId) =>
      set((state) => {
        state.activeDeviceId = deviceId;
      }),

    setStatus: (status) =>
      set((state) => {
        state.status = status;
      }),

    setLoading: (isLoading) =>
      set((state) => {
        state.isLoading = isLoading;
      }),

    setError: (error) =>
      set((state) => {
        state.error = error;
      }),

    updateDevice: (deviceId, updates) =>
      set((state) => {
        const index = state.devices.findIndex((d) => d.id === deviceId);
        if (index !== -1) {
          Object.assign(state.devices[index], updates);
        }
      }),
  }))
);
