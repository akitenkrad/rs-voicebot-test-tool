//! Virtual device management module
//!
//! Handles creation and management of virtual audio devices
//! with platform-specific implementations.

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "windows")]
pub mod windows;

use crate::error::DeviceError;
use crate::types::{DeviceInfo, DeviceStatus};

/// Platform-agnostic virtual device manager trait.
///
/// Each platform provides its own implementation:
/// - macOS: detects BlackHole / Soundflower (cannot create programmatically)
/// - Windows: detects VB-Audio Virtual Cable / VoiceMeeter
/// - Linux: creates/destroys PulseAudio null-sink modules via `pactl`
pub trait VirtualDeviceManager: Send + Sync {
    /// Create a virtual audio device (or detect an existing one like BlackHole/VB-Cable).
    fn create_device(&mut self, name: &str) -> Result<DeviceInfo, DeviceError>;

    /// Destroy/cleanup the virtual device.
    fn destroy_device(&mut self) -> Result<(), DeviceError>;

    /// List all audio output devices on the system.
    fn list_devices(&self) -> Result<Vec<DeviceInfo>, DeviceError>;

    /// Check if a virtual audio device (BlackHole, VB-Cable, PulseAudio sink) is available.
    fn detect_virtual_device(&self) -> Result<Option<DeviceInfo>, DeviceError>;

    /// Get the current device status.
    fn get_status(&self) -> Result<DeviceStatus, DeviceError>;

    /// Get the name of the currently active virtual device.
    fn active_device_name(&self) -> Option<String>;
}

/// Create the appropriate platform-specific device manager.
pub fn create_device_manager() -> Box<dyn VirtualDeviceManager> {
    #[cfg(target_os = "macos")]
    {
        Box::new(macos::MacOSDeviceManager::new())
    }

    #[cfg(target_os = "windows")]
    {
        Box::new(windows::WindowsDeviceManager::new())
    }

    #[cfg(target_os = "linux")]
    {
        Box::new(linux::LinuxDeviceManager::new())
    }
}
