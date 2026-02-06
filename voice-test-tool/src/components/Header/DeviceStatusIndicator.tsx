import { useEffect, useCallback, useState } from "react";
import { useTranslation } from "react-i18next";
import { useDevice } from "../../hooks/useDevice";
import { useConfig } from "../../hooks/useConfig";
import { cn } from "../../lib/utils";

export function DeviceStatusIndicator() {
  const { t } = useTranslation();
  const {
    status,
    activeDeviceId,
    isLoading,
    error: deviceError,
    refreshStatus,
    createDevice,
    destroyDevice,
  } = useDevice();
  const { config, fetchConfig } = useConfig();

  // Local error state for transient create/destroy errors
  const [lastError, setLastError] = useState<string | null>(null);

  // Fetch device status and config on mount
  useEffect(() => {
    refreshStatus().catch((err) => {
      console.log("[DeviceStatusIndicator] initial status fetch error:", err);
    });
    fetchConfig().catch(() => {
      // Config fetch failure is non-critical here
    });
  }, [refreshStatus, fetchConfig]);

  // Clear transient error after 5 seconds
  useEffect(() => {
    if (!lastError) return;
    const timer = setTimeout(() => setLastError(null), 5000);
    return () => clearTimeout(timer);
  }, [lastError]);

  const isActive = status?.is_active ?? false;
  const displayError = lastError || deviceError;
  const deviceName = status?.device_name || t("header.virtualMic");

  const dotColor = displayError
    ? "bg-device-error"
    : isActive
      ? "bg-device-active"
      : "bg-device-inactive";

  /** Toggle device: create if none active, destroy if one is active */
  const handleClick = useCallback(async () => {
    if (isLoading) return;
    setLastError(null);

    try {
      if (activeDeviceId) {
        await destroyDevice();
      } else {
        // Use device name from config, or a sensible default
        const name = config?.virtual_device?.device_name || "VoiceTestTool Virtual Mic";
        await createDevice(name);
      }
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setLastError(message);
      console.error("[DeviceStatusIndicator] toggle device error:", message);
    }
  }, [isLoading, activeDeviceId, createDevice, destroyDevice, config]);

  return (
    <div className="flex flex-col items-end gap-1">
      <button
        type="button"
        onClick={handleClick}
        disabled={isLoading}
        className={cn(
          "flex items-center gap-2 rounded-md border border-border bg-background px-3 py-1.5 transition-colors hover:bg-accent",
          isLoading && "opacity-50 cursor-wait"
        )}
      >
        <span
          className={cn("h-2.5 w-2.5 rounded-full", dotColor)}
          aria-hidden="true"
        />
        <span className="text-sm font-medium text-foreground">{deviceName}</span>
        <span className="text-xs text-muted-foreground">
          {isActive ? t("header.deviceActive") : t("header.deviceInactive")}
        </span>
      </button>
      {/* Show error message below the button when present */}
      {displayError && (
        <span className="max-w-[320px] truncate text-xs text-red-500" title={displayError}>
          {displayError}
        </span>
      )}
    </div>
  );
}
