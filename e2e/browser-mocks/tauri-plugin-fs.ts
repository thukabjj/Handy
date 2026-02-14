/**
 * Mock for @tauri-apps/plugin-fs
 */
import { mockState } from "./mock-state";

export async function readTextFile(path: string): Promise<string> {
  return mockState.fileSystem[path] ?? "";
}

export async function writeTextFile(path: string, content: string): Promise<void> {
  mockState.fileSystem[path] = content;
}

export async function readFile(path: string): Promise<Uint8Array> {
  const text = mockState.fileSystem[path] ?? "";
  return new TextEncoder().encode(text);
}

export async function writeFile(path: string, data: Uint8Array): Promise<void> {
  mockState.fileSystem[path] = new TextDecoder().decode(data);
}

export async function exists(path: string): Promise<boolean> {
  return path in mockState.fileSystem;
}

export async function mkdir(_path: string, _options?: unknown): Promise<void> {}

export async function remove(_path: string, _options?: unknown): Promise<void> {}

export async function readDir(_path: string): Promise<Array<{ name: string; isDirectory: boolean; isFile: boolean }>> {
  return [];
}

// Re-export BaseDirectory enum stub
export const BaseDirectory = {
  Audio: 1,
  Cache: 2,
  Config: 3,
  Data: 4,
  LocalData: 5,
  Document: 6,
  Download: 7,
  Picture: 8,
  Public: 9,
  Video: 10,
  Resource: 11,
  Temp: 12,
  AppConfig: 13,
  AppData: 14,
  AppLocalData: 15,
  AppCache: 16,
  AppLog: 17,
  Desktop: 18,
  Executable: 19,
  Font: 20,
  Home: 21,
  Runtime: 22,
  Template: 23,
} as const;
