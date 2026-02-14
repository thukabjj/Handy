/**
 * Mock for @tauri-apps/plugin-opener
 */

export async function openUrl(url: string): Promise<void> {
  console.log("[E2E Mock] openUrl:", url);
}

export async function openPath(path: string): Promise<void> {
  console.log("[E2E Mock] openPath:", path);
}
