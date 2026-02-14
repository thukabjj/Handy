/**
 * Mock for @tauri-apps/plugin-os
 *
 * CRITICAL: platform() is called SYNCHRONOUSLY in App.tsx:106.
 * It must return a string directly, not a Promise.
 * The type() function is also synchronous per useOsType.ts.
 */

/** Returns "macos" | "windows" | "linux" | "ios" | "android" */
export function platform(): string {
  return "macos";
}

/** Returns "Darwin" | "Windows_NT" | "Linux" */
export function type(): string {
  return "macos";
}

/** Returns OS version string */
export function version(): string {
  return "14.0";
}

/** Returns system locale string (async) */
export async function locale(): Promise<string | null> {
  return "en-US";
}

export function arch(): string {
  return "aarch64";
}

export function family(): string {
  return "unix";
}

export function hostname(): Promise<string> {
  return Promise.resolve("mock-host");
}

export function eol(): string {
  return "\n";
}

export function exeExtension(): string {
  return "";
}
