/**
 * Mock for @tauri-apps/plugin-autostart
 */
import { mockState } from "./mock-state";

export async function enable(): Promise<void> {
  mockState.autostart = true;
}

export async function disable(): Promise<void> {
  mockState.autostart = false;
}

export async function isEnabled(): Promise<boolean> {
  return mockState.autostart;
}
