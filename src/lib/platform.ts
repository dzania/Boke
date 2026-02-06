/**
 * Platform utilities that work in both Tauri (desktop) and web modes.
 * Provides fallbacks for Tauri-specific functionality.
 */

// Runtime detection (evaluated at call time, not module load)
// Tauri v2 uses __TAURI_INTERNALS__, v1 used __TAURI__
function checkIsTauri(): boolean {
  return (
    typeof window !== "undefined" && ("__TAURI_INTERNALS__" in window || "__TAURI__" in window)
  );
}

/**
 * Open a URL in the default browser/handler.
 * Uses Tauri's opener plugin in desktop mode, window.open in web mode.
 */
export async function openUrl(url: string): Promise<void> {
  if (checkIsTauri()) {
    const { openUrl: tauriOpenUrl } = await import("@tauri-apps/plugin-opener");
    return tauriOpenUrl(url);
  } else {
    window.open(url, "_blank", "noopener,noreferrer");
  }
}

/**
 * Listen for events from the backend.
 * In web mode, this is a no-op since we don't have Tauri events.
 */
export async function listen<T>(
  event: string,
  handler: (event: { payload: T }) => void
): Promise<() => void> {
  if (checkIsTauri()) {
    const { listen: tauriListen } = await import("@tauri-apps/api/event");
    return tauriListen(event, handler);
  } else {
    // No-op in web mode - return empty cleanup function
    return () => {};
  }
}

/**
 * Check if notification permission is granted.
 */
export async function isNotificationPermissionGranted(): Promise<boolean> {
  if (checkIsTauri()) {
    const { isPermissionGranted } = await import("@tauri-apps/plugin-notification");
    return isPermissionGranted();
  } else {
    return Notification.permission === "granted";
  }
}

/**
 * Request notification permission.
 */
export async function requestNotificationPermission(): Promise<boolean> {
  if (checkIsTauri()) {
    const { requestPermission } = await import("@tauri-apps/plugin-notification");
    const permission = await requestPermission();
    return permission === "granted";
  } else {
    const permission = await Notification.requestPermission();
    return permission === "granted";
  }
}

/**
 * Send a notification.
 */
export async function sendNotification(title: string, body: string): Promise<void> {
  if (checkIsTauri()) {
    const { sendNotification: tauriNotify } = await import("@tauri-apps/plugin-notification");
    tauriNotify({ title, body });
  } else {
    new Notification(title, { body });
  }
}

/**
 * Check if running in desktop (Tauri) mode.
 */
export function isDesktop(): boolean {
  return checkIsTauri();
}

/**
 * Check if running in web mode.
 */
export function isWeb(): boolean {
  return !checkIsTauri();
}

/**
 * Open a file picker dialog.
 * In Tauri mode, uses the native file dialog.
 * In web mode, creates a hidden file input.
 * Returns the file path (Tauri) or File object (web).
 */
export async function openFileDialog(options?: {
  filters?: Array<{ name: string; extensions: string[] }>;
}): Promise<string | File | null> {
  if (checkIsTauri()) {
    const { open } = await import("@tauri-apps/plugin-dialog");
    const result = await open({
      multiple: false,
      filters: options?.filters,
    });
    return result; // Returns string path or null
  } else {
    // Create a file input and trigger it
    return new Promise((resolve) => {
      const input = document.createElement("input");
      input.type = "file";
      if (options?.filters && options.filters.length > 0) {
        const extensions = options.filters.flatMap((f) => f.extensions.map((ext) => `.${ext}`));
        input.accept = extensions.join(",");
      }
      input.onchange = () => {
        const file = input.files?.[0] ?? null;
        resolve(file);
      };
      input.click();
    });
  }
}
