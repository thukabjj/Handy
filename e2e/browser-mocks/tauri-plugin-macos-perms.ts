/**
 * Mock for tauri-plugin-macos-permissions-api
 * All permissions return true (granted) by default.
 *
 * Important: These must use real async delays (not just `async` keyword)
 * because AccessibilityOnboarding.tsx has `onComplete` in its useEffect
 * dependency array. With instant resolution, the effect re-triggers before
 * React can unmount the component, causing an infinite update loop.
 */

const delay = (ms: number) => new Promise((r) => setTimeout(r, ms));

export async function checkAccessibilityPermission(): Promise<boolean> {
  await delay(10);
  return true;
}

export async function requestAccessibilityPermission(): Promise<boolean> {
  await delay(10);
  return true;
}

export async function checkMicrophonePermission(): Promise<boolean> {
  await delay(10);
  return true;
}

export async function requestMicrophonePermission(): Promise<boolean> {
  await delay(10);
  return true;
}

export async function checkScreenCapturePermission(): Promise<boolean> {
  await delay(10);
  return true;
}

export async function requestScreenCapturePermission(): Promise<boolean> {
  await delay(10);
  return true;
}
