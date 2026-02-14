/**
 * Mock for @tauri-apps/plugin-process
 */

export async function relaunch(): Promise<void> {
  console.log("[E2E Mock] relaunch requested — reloading page");
  window.location.reload();
}

export async function exit(_exitCode?: number): Promise<void> {
  console.log("[E2E Mock] exit requested");
}
