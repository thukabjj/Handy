/**
 * Mock for @tauri-apps/api/webviewWindow
 */

export class WebviewWindow {
  label: string;

  constructor(label: string, _options?: unknown) {
    this.label = label;
  }

  async listen(_event: string, _handler: unknown): Promise<() => void> {
    return () => {};
  }

  async once(_event: string, _handler: unknown): Promise<() => void> {
    return () => {};
  }

  async emit(_event: string, _payload?: unknown): Promise<void> {}
  async close(): Promise<void> {}
  async show(): Promise<void> {}
  async hide(): Promise<void> {}
  async setFocus(): Promise<void> {}
}
