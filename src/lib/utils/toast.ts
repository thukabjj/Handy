import { toast } from "sonner";

/**
 * Toast utility functions for consistent feedback throughout the app
 */

/**
 * Show a success toast
 */
export const showSuccessToast = (message: string) => {
  toast.success(message, {
    duration: 2000,
  });
};

/**
 * Options for error toasts
 */
export interface ErrorToastOptions {
  /** Additional description shown below the main message */
  description?: string;
  /** Action button configuration */
  action?: {
    label: string;
    onClick: () => void;
  };
  /** Duration in milliseconds (default: 5000) */
  duration?: number;
}

/**
 * Show an error toast with optional description and action
 */
export const showErrorToast = (
  message: string,
  options?: ErrorToastOptions
) => {
  toast.error(message, {
    duration: options?.duration ?? 5000,
    description: options?.description,
    action: options?.action
      ? {
          label: options.action.label,
          onClick: options.action.onClick,
        }
      : undefined,
  });
};

/**
 * Show an info toast
 */
export const showInfoToast = (message: string) => {
  toast.info(message, {
    duration: 3000,
  });
};

/**
 * Show a loading toast that can be updated
 */
export const showLoadingToast = (message: string) => {
  return toast.loading(message);
};

/**
 * Dismiss a specific toast by id
 */
export const dismissToast = (toastId: string | number) => {
  toast.dismiss(toastId);
};
