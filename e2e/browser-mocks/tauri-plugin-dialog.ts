/**
 * Mock for @tauri-apps/plugin-dialog
 */

export async function ask(
  message: string,
  _options?: unknown,
): Promise<boolean> {
  console.log("[E2E Mock] dialog.ask:", message);
  return true;
}

export async function confirm(
  message: string,
  _options?: unknown,
): Promise<boolean> {
  console.log("[E2E Mock] dialog.confirm:", message);
  return true;
}

export async function message(
  msg: string,
  _options?: unknown,
): Promise<void> {
  console.log("[E2E Mock] dialog.message:", msg);
}

export async function open(
  _options?: unknown,
): Promise<string | string[] | null> {
  return "/mock/selected-file.txt";
}

export async function save(
  _options?: unknown,
): Promise<string | null> {
  return "/mock/saved-file.txt";
}
