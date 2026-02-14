/**
 * Mock for @tauri-apps/plugin-updater
 */

export async function check(): Promise<{
  available: boolean;
  currentVersion: string;
  version: string;
  date: string | null;
  body: string | null;
} | null> {
  return null;
}
