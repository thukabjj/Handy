/**
 * Mock for @tauri-apps/api/window
 */

function noop() {}
function noopAsync() { return Promise.resolve(); }

const mockWindow = {
  close: noopAsync,
  hide: noopAsync,
  show: noopAsync,
  setFocus: noopAsync,
  minimize: noopAsync,
  maximize: noopAsync,
  unmaximize: noopAsync,
  center: noopAsync,
  setTitle: noopAsync,
  setSize: noopAsync,
  setPosition: noopAsync,
  setAlwaysOnTop: noopAsync,
  setDecorations: noopAsync,
  setResizable: noopAsync,
  setFullscreen: noopAsync,
  isFullscreen: () => Promise.resolve(false),
  isMaximized: () => Promise.resolve(false),
  isMinimized: () => Promise.resolve(false),
  isVisible: () => Promise.resolve(true),
  listen: (_event: string, _handler: unknown) => Promise.resolve(noop),
  once: (_event: string, _handler: unknown) => Promise.resolve(noop),
  emit: noopAsync,
};

export function getCurrentWindow() {
  return mockWindow;
}

export function getCurrent() {
  return mockWindow;
}
