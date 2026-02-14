/**
 * Mock for @tauri-apps/plugin-clipboard-manager
 */
import { mockState } from "./mock-state";

export async function writeText(text: string): Promise<void> {
  mockState.clipboard = text;
}

export async function readText(): Promise<string> {
  return mockState.clipboard;
}
