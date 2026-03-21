import { invoke } from "@tauri-apps/api/core";

export type FrontendTelemetryLevel =
  | "trace"
  | "debug"
  | "info"
  | "warn"
  | "error";

export interface FrontendTelemetryAttribute {
  key: string;
  value: string;
  sensitive?: boolean;
}

export interface FrontendTelemetryEventInput {
  component: string;
  action: string;
  message: string;
  level?: FrontendTelemetryLevel;
  traceId?: string;
  sessionId?: string;
  durationMs?: number;
  status?: string;
  attributes?: FrontendTelemetryAttribute[];
}

export const newTraceId = (): string => {
  if (typeof crypto !== "undefined" && "randomUUID" in crypto) {
    return crypto.randomUUID();
  }

  return `trace_${Date.now()}_${Math.random().toString(36).slice(2, 10)}`;
};

export const recordFrontendTelemetryEvent = async (
  event: FrontendTelemetryEventInput,
): Promise<void> => {
  try {
    await invoke("record_frontend_telemetry_event", {
      event: {
        component: `frontend.${event.component}`,
        action: event.action,
        level: event.level ?? "info",
        message: event.message,
        trace_id: event.traceId ?? null,
        session_id: event.sessionId ?? null,
        duration_ms: event.durationMs ?? null,
        status: event.status ?? null,
        source: "frontend",
        attributes:
          event.attributes?.map((attribute) => ({
            key: attribute.key,
            value: attribute.value,
            sensitive: attribute.sensitive ?? false,
          })) ?? [],
      },
    });
  } catch (error) {
    console.warn("Failed to record frontend telemetry event:", error);
  }
};
