import { create } from "zustand";
import {
  AppError,
  ErrorCategory,
  HandyError,
  toAppError,
  normalizeError,
} from "@/lib/errors/types";
import { showErrorToast } from "@/lib/utils/toast";

const MAX_ERROR_HISTORY = 50;

interface ErrorStore {
  /**
   * List of all errors (kept for debugging/history)
   */
  errors: AppError[];

  /**
   * Optional handler for retrying the last failed operation
   */
  retryLastOperation: (() => Promise<void>) | null;

  /**
   * Add an error to the store and display a toast notification
   */
  addError: (error: HandyError | string | unknown, context?: string) => AppError;

  /**
   * Dismiss a specific error by ID
   */
  dismissError: (id: string) => void;

  /**
   * Dismiss all active errors
   */
  dismissAllErrors: () => void;

  /**
   * Get errors filtered by category
   */
  getErrorsByCategory: (category: ErrorCategory) => AppError[];

  /**
   * Check if there are any active (non-dismissed) errors
   */
  hasActiveErrors: () => boolean;

  /**
   * Get the count of active errors
   */
  getActiveErrorCount: () => number;

  /**
   * Set a retry handler for recoverable errors
   */
  setRetryHandler: (handler: (() => Promise<void>) | null) => void;

  /**
   * Clear all errors from history
   */
  clearErrorHistory: () => void;
}

export const useErrorStore = create<ErrorStore>((set, get) => ({
  errors: [],
  retryLastOperation: null,

  addError: (error, context) => {
    // Normalize the error to HandyError format
    const handyError = normalizeError(error);

    // Convert to AppError with metadata
    const appError = toAppError(handyError, context);

    // Show toast notification
    const fullMessage = context
      ? `${context}: ${handyError.message}`
      : handyError.message;

    showErrorToast(fullMessage, {
      description: handyError.suggestion || handyError.details,
      action:
        handyError.recoverable && get().retryLastOperation
          ? {
              label: "Retry",
              onClick: () => {
                const retry = get().retryLastOperation;
                if (retry) {
                  retry().catch((e) => {
                    console.error("Retry failed:", e);
                  });
                }
              },
            }
          : undefined,
    });

    // Add to error history (keeping last MAX_ERROR_HISTORY errors)
    set((state) => ({
      errors: [...state.errors.slice(-(MAX_ERROR_HISTORY - 1)), appError],
    }));

    // Log to console for debugging
    console.error("[AppError]", {
      category: appError.category,
      message: appError.message,
      details: appError.details,
      context: appError.context,
    });

    return appError;
  },

  dismissError: (id) => {
    set((state) => ({
      errors: state.errors.map((e) =>
        e.id === id ? { ...e, dismissed: true } : e
      ),
    }));
  },

  dismissAllErrors: () => {
    set((state) => ({
      errors: state.errors.map((e) => ({ ...e, dismissed: true })),
    }));
  },

  getErrorsByCategory: (category) => {
    return get().errors.filter(
      (e) => e.category === category && !e.dismissed
    );
  },

  hasActiveErrors: () => {
    return get().errors.some((e) => !e.dismissed);
  },

  getActiveErrorCount: () => {
    return get().errors.filter((e) => !e.dismissed).length;
  },

  setRetryHandler: (handler) => {
    set({ retryLastOperation: handler });
  },

  clearErrorHistory: () => {
    set({ errors: [] });
  },
}));

/**
 * Helper function to handle errors from Tauri command results
 * Use this when calling Tauri commands that may return errors
 */
export function handleCommandError(
  error: unknown,
  context: string
): void {
  useErrorStore.getState().addError(error, context);
}

/**
 * Helper to create an error handler with a specific context
 * Useful for wrapping async operations
 */
export function createErrorHandler(context: string) {
  return (error: unknown) => {
    handleCommandError(error, context);
  };
}
