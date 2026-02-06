/** Platform type for virtual device management */
export type Platform = "linux" | "macos" | "windows";

/**
 * Information about an audio device, matching the Rust DeviceInfo struct.
 *
 * Rust fields: id, name, is_virtual, is_default, is_active
 */
export interface DeviceInfo {
  id: string;
  name: string;
  is_virtual: boolean;
  is_default: boolean;
  is_active: boolean;
}

/**
 * Current status of the virtual device, matching the Rust DeviceStatus struct.
 *
 * Rust fields: device_name, is_active, is_default, error (Option<String>)
 * NOTE: The Rust struct does NOT include `platform`.
 */
export interface DeviceStatus {
  device_name: string;
  is_active: boolean;
  is_default: boolean;
  error?: string;
}
