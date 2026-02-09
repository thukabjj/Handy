/**
 * Error categories matching the backend HandyError type
 */
export type ErrorCategory =
  | "settings"
  | "audio"
  | "model"
  | "transcription"
  | "network"
  | "validation"
  | "state"
  | "filesystem"
  | "permission"
  | "unknown";

/**
 * Structured error type from the backend
 * This matches the Rust HandyError struct
 */
export interface HandyError {
  category: ErrorCategory;
  message: string;
  details?: string;
  recoverable: boolean;
  suggestion?: string;
}

/**
 * Extended error type for frontend use with tracking metadata
 */
export interface AppError extends HandyError {
  id: string;
  timestamp: Date;
  dismissed: boolean;
  context?: string;
}

/**
 * Convert a HandyError (from backend) to an AppError (for frontend use)
 */
export function toAppError(
  error: HandyError,
  context?: string
): AppError {
  return {
    ...error,
    id: generateErrorId(),
    timestamp: new Date(),
    dismissed: false,
    context,
  };
}

/**
 * Convert a string error to a HandyError
 */
export function stringToHandyError(message: string): HandyError {
  return {
    category: "unknown",
    message,
    recoverable: false,
  };
}

/**
 * Check if an error response is a HandyError
 */
export function isHandyError(error: unknown): error is HandyError {
  if (typeof error !== "object" || error === null) {
    return false;
  }

  const obj = error as Record<string, unknown>;
  return (
    typeof obj.category === "string" &&
    typeof obj.message === "string" &&
    typeof obj.recoverable === "boolean"
  );
}

/**
 * Normalize any error to a HandyError
 */
export function normalizeError(error: unknown): HandyError {
  if (isHandyError(error)) {
    return error;
  }

  if (error instanceof Error) {
    return stringToHandyError(error.message);
  }

  if (typeof error === "string") {
    return stringToHandyError(error);
  }

  return stringToHandyError("An unknown error occurred");
}

/**
 * Generate a unique error ID
 */
function generateErrorId(): string {
  return `err_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
}

/**
 * Get a user-friendly message for an error category
 */
export function getCategoryDisplayName(category: ErrorCategory): string {
  const displayNames: Record<ErrorCategory, string> = {
    settings: "Settings",
    audio: "Audio",
    model: "Model",
    transcription: "Transcription",
    network: "Network",
    validation: "Validation",
    state: "Application State",
    filesystem: "File System",
    permission: "Permission",
    unknown: "Error",
  };

  return displayNames[category] || "Error";
}
