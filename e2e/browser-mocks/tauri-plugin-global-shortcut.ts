/**
 * Mock for @tauri-apps/plugin-global-shortcut
 * Stub — global shortcuts are handled natively and don't affect browser E2E tests.
 */

export async function register(
  _shortcut: string,
  _handler: () => void,
): Promise<void> {}

export async function unregister(_shortcut: string): Promise<void> {}

export async function unregisterAll(): Promise<void> {}

export async function isRegistered(_shortcut: string): Promise<boolean> {
  return false;
}
