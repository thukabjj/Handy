/**
 * Mock for @tauri-apps/api/event
 * Stores callbacks in mockState.eventListeners Map.
 */
import { mockState } from "./mock-state";

type EventCallback<T = unknown> = (event: {
  event: string;
  id: number;
  windowLabel: string;
  payload: T;
}) => void;

type UnlistenFn = () => void;

export async function listen<T = unknown>(
  event: string,
  handler: EventCallback<T>,
): Promise<UnlistenFn> {
  if (!mockState.eventListeners.has(event)) {
    mockState.eventListeners.set(event, new Set());
  }
  mockState.eventListeners.get(event)!.add(handler as EventCallback);

  return () => {
    const listeners = mockState.eventListeners.get(event);
    if (listeners) {
      listeners.delete(handler as EventCallback);
    }
  };
}

export async function once<T = unknown>(
  event: string,
  handler: EventCallback<T>,
): Promise<UnlistenFn> {
  const wrappedHandler: EventCallback<T> = (e) => {
    handler(e);
    const listeners = mockState.eventListeners.get(event);
    if (listeners) {
      listeners.delete(wrappedHandler as EventCallback);
    }
  };

  if (!mockState.eventListeners.has(event)) {
    mockState.eventListeners.set(event, new Set());
  }
  mockState.eventListeners.get(event)!.add(wrappedHandler as EventCallback);

  return () => {
    const listeners = mockState.eventListeners.get(event);
    if (listeners) {
      listeners.delete(wrappedHandler as EventCallback);
    }
  };
}

export async function emit(event: string, payload?: unknown): Promise<void> {
  const listeners = mockState.eventListeners.get(event);
  if (listeners) {
    for (const cb of listeners) {
      try {
        cb({
          event,
          id: Math.random(),
          windowLabel: "main",
          payload,
        });
      } catch (e) {
        console.warn(`[E2E Mock] Error emitting event "${event}":`, e);
      }
    }
  }
}
