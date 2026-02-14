/**
 * Mock for @tauri-apps/api/app
 */

export async function getVersion(): Promise<string> {
  return "0.8.0";
}

export async function getName(): Promise<string> {
  return "Dictum";
}

export async function getTauriVersion(): Promise<string> {
  return "2.9.0";
}
