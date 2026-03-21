import React, { useEffect, useMemo, useState } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
import { Button } from "../../ui/Button";
import { recordFrontendTelemetryEvent } from "@/lib/telemetry";

type ObservabilityDataMode =
  | "metadata_only"
  | "sensitive_text"
  | "developer";

type TelemetryLevel = "trace" | "debug" | "info" | "warn" | "error";

interface ObservabilitySettings {
  enabled: boolean;
  data_mode: ObservabilityDataMode;
  max_events: number;
  emit_live_events: boolean;
  capture_frontend_events: boolean;
}

interface TelemetryAttribute {
  key: string;
  value: string;
  redacted?: boolean;
}

interface TelemetryEvent {
  id: string;
  timestamp_ms: number;
  component: string;
  action: string;
  level: TelemetryLevel;
  message: string;
  trace_id?: string | null;
  session_id?: string | null;
  duration_ms?: number | null;
  status?: string | null;
  source: string;
  attributes: TelemetryAttribute[];
}

interface TelemetryComponentStat {
  component: string;
  total_events: number;
  error_events: number;
  warn_events: number;
  last_event_at_ms?: number | null;
}

interface TelemetryStats {
  total_events: number;
  buffered_events: number;
  dropped_events: number;
  error_events: number;
  warn_events: number;
  last_event_at_ms?: number | null;
  live_streaming_enabled: boolean;
  component_breakdown: TelemetryComponentStat[];
}

interface TelemetrySnapshot {
  settings: ObservabilitySettings;
  stats: TelemetryStats;
  events: TelemetryEvent[];
}

const MODE_OPTIONS: Array<{
  value: ObservabilityDataMode;
  label: string;
  description: string;
}> = [
  {
    value: "metadata_only",
    label: "Metadata only",
    description: "Safe default. Content fields are redacted.",
  },
  {
    value: "sensitive_text",
    label: "Sensitive text",
    description: "Includes transcripts and prompts in the diagnostics stream.",
  },
  {
    value: "developer",
    label: "Developer",
    description: "Maximum local diagnostics detail.",
  },
];

const levelClasses: Record<TelemetryLevel, string> = {
  trace: "border-mid-gray/20 text-mid-gray",
  debug: "border-sky-500/30 text-sky-300",
  info: "border-emerald-500/30 text-emerald-300",
  warn: "border-amber-500/30 text-amber-300",
  error: "border-red-500/30 text-red-300",
};

const formatTimestamp = (timestampMs?: number | null): string => {
  if (!timestampMs) {
    return "n/a";
  }

  return new Date(timestampMs).toLocaleTimeString([], {
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  });
};

export const DiagnosticsSettings: React.FC = () => {
  const { t } = useTranslation();
  const [settings, setSettings] = useState<ObservabilitySettings | null>(null);
  const [stats, setStats] = useState<TelemetryStats | null>(null);
  const [events, setEvents] = useState<TelemetryEvent[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [search, setSearch] = useState("");
  const [selectedComponent, setSelectedComponent] = useState("all");
  const [selectedLevel, setSelectedLevel] = useState<"all" | TelemetryLevel>(
    "all",
  );
  const [livePaused, setLivePaused] = useState(false);
  const [exportPath, setExportPath] = useState<string | null>(null);

  const refreshSnapshot = async () => {
    setIsLoading(true);
    try {
      const snapshot = await invoke<TelemetrySnapshot>("get_telemetry_snapshot");
      setSettings(snapshot.settings);
      setStats(snapshot.stats);
      setEvents(snapshot.events);
    } catch (error) {
      console.error("Failed to load telemetry snapshot:", error);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    let unlisten: UnlistenFn | null = null;

    const setup = async () => {
      await refreshSnapshot();
      recordFrontendTelemetryEvent({
        component: "diagnostics",
        action: "panel_opened",
        message: "Diagnostics panel opened",
      });

      unlisten = await listen<TelemetryEvent>("telemetry-event", (event) => {
        if (livePaused) {
          return;
        }

        const nextEvent = event.payload;
        setEvents((previous) => {
          const next = [...previous, nextEvent];
          return next.length > 2000 ? next.slice(next.length - 2000) : next;
        });
        setStats((previous) => {
          if (!previous) {
            return previous;
          }

          const componentMap = new Map(
            previous.component_breakdown.map((item) => [item.component, item]),
          );
          const component = componentMap.get(nextEvent.component) ?? {
            component: nextEvent.component,
            total_events: 0,
            error_events: 0,
            warn_events: 0,
            last_event_at_ms: null,
          };

          component.total_events += 1;
          component.last_event_at_ms = nextEvent.timestamp_ms;
          if (nextEvent.level === "error") {
            component.error_events += 1;
          }
          if (nextEvent.level === "warn") {
            component.warn_events += 1;
          }
          componentMap.set(nextEvent.component, component);

          const component_breakdown = [...componentMap.values()].sort(
            (left, right) =>
              right.total_events - left.total_events ||
              left.component.localeCompare(right.component),
          );

          return {
            ...previous,
            total_events: previous.total_events + 1,
            buffered_events: previous.buffered_events + 1,
            error_events:
              previous.error_events + (nextEvent.level === "error" ? 1 : 0),
            warn_events:
              previous.warn_events + (nextEvent.level === "warn" ? 1 : 0),
            last_event_at_ms: nextEvent.timestamp_ms,
            component_breakdown,
          };
        });
      });
    };

    setup();

    return () => {
      if (unlisten) {
        unlisten();
      }
    };
  }, [livePaused]);

  const componentOptions = useMemo(() => {
    const unique = new Set(events.map((event) => event.component));
    return ["all", ...[...unique].sort()];
  }, [events]);

  const filteredEvents = useMemo(() => {
    const query = search.trim().toLowerCase();
    return [...events]
      .reverse()
      .filter((event) => {
        if (selectedComponent !== "all" && event.component !== selectedComponent) {
          return false;
        }
        if (selectedLevel !== "all" && event.level !== selectedLevel) {
          return false;
        }
        if (!query) {
          return true;
        }

        const haystack = [
          event.component,
          event.action,
          event.message,
          event.status ?? "",
          event.session_id ?? "",
          event.trace_id ?? "",
          ...event.attributes.flatMap((attribute) => [
            attribute.key,
            attribute.value,
          ]),
        ]
          .join(" ")
          .toLowerCase();

        return haystack.includes(query);
      });
  }, [events, search, selectedComponent, selectedLevel]);

  const persistSettings = async (nextSettings: ObservabilitySettings) => {
    setSettings(nextSettings);
    try {
      await invoke("update_observability_settings", { settings: nextSettings });
      await refreshSnapshot();
      recordFrontendTelemetryEvent({
        component: "diagnostics",
        action: "settings_updated",
        message: "Observability settings updated",
        attributes: [
          { key: "enabled", value: String(nextSettings.enabled) },
          { key: "data_mode", value: nextSettings.data_mode },
          { key: "max_events", value: String(nextSettings.max_events) },
        ],
      });
    } catch (error) {
      console.error("Failed to update observability settings:", error);
      await refreshSnapshot();
    }
  };

  const clearEvents = async () => {
    try {
      await invoke("clear_telemetry_events");
      setEvents([]);
      await refreshSnapshot();
      recordFrontendTelemetryEvent({
        component: "diagnostics",
        action: "events_cleared",
        message: "Diagnostics event buffer cleared",
      });
    } catch (error) {
      console.error("Failed to clear telemetry events:", error);
    }
  };

  const exportBundle = async () => {
    try {
      const path = await invoke<string>("export_telemetry_snapshot_bundle");
      setExportPath(path);
      recordFrontendTelemetryEvent({
        component: "diagnostics",
        action: "bundle_exported",
        message: "Observability bundle exported",
        attributes: [{ key: "path", value: path }],
      });
    } catch (error) {
      console.error("Failed to export telemetry bundle:", error);
    }
  };

  return (
    <div className="space-y-4">
      <div className="rounded-lg border border-mid-gray/20 bg-mid-gray/5 p-4">
        <div className="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
          <div>
            <h3 className="text-sm font-semibold">
              {t("diagnostics.title", "Observability")}
            </h3>
            <p className="mt-1 max-w-2xl text-xs text-mid-gray">
              {t(
                "diagnostics.description",
                "Structured diagnostics across backend, overlay, wake pipeline, and provider requests. Use the data mode carefully because sensitive text can be captured locally.",
              )}
            </p>
          </div>
          <div className="flex flex-wrap gap-2">
            <Button
              variant="secondary"
              size="sm"
              onClick={() => void refreshSnapshot()}
            >
              {t("common.refresh", "Refresh")}
            </Button>
            <Button variant="secondary" size="sm" onClick={() => void exportBundle()}>
              Export bundle
            </Button>
            <Button variant="danger-ghost" size="sm" onClick={() => void clearEvents()}>
              {t("common.clear", "Clear")}
            </Button>
          </div>
        </div>
        {exportPath && (
          <p className="mt-3 text-xs text-mid-gray">
            Latest bundle: <span className="font-mono text-text">{exportPath}</span>
          </p>
        )}
      </div>

      <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-4">
        <div className="rounded-lg border border-mid-gray/20 bg-background/50 p-3">
          <div className="text-xs uppercase tracking-wide text-mid-gray">
            Buffered
          </div>
          <div className="mt-2 text-2xl font-semibold">
            {stats?.buffered_events ?? 0}
          </div>
        </div>
        <div className="rounded-lg border border-mid-gray/20 bg-background/50 p-3">
          <div className="text-xs uppercase tracking-wide text-mid-gray">
            Errors
          </div>
          <div className="mt-2 text-2xl font-semibold text-red-300">
            {stats?.error_events ?? 0}
          </div>
        </div>
        <div className="rounded-lg border border-mid-gray/20 bg-background/50 p-3">
          <div className="text-xs uppercase tracking-wide text-mid-gray">
            Warnings
          </div>
          <div className="mt-2 text-2xl font-semibold text-amber-300">
            {stats?.warn_events ?? 0}
          </div>
        </div>
        <div className="rounded-lg border border-mid-gray/20 bg-background/50 p-3">
          <div className="text-xs uppercase tracking-wide text-mid-gray">
            Dropped
          </div>
          <div className="mt-2 text-2xl font-semibold">
            {stats?.dropped_events ?? 0}
          </div>
        </div>
      </div>

      <div className="rounded-lg border border-mid-gray/20 bg-background/40 p-4">
        <div className="grid gap-4 lg:grid-cols-[1fr_1fr_160px_160px]">
          <label className="space-y-2 text-sm">
            <span className="block text-xs uppercase tracking-wide text-mid-gray">
              Mode
            </span>
            <select
              className="w-full rounded-md border border-mid-gray/20 bg-background px-3 py-2 text-sm"
              value={settings?.data_mode ?? "metadata_only"}
              onChange={(event) => {
                if (!settings) return;
                void persistSettings({
                  ...settings,
                  data_mode: event.target.value as ObservabilityDataMode,
                });
              }}
            >
              {MODE_OPTIONS.map((option) => (
                <option key={option.value} value={option.value}>
                  {option.label}
                </option>
              ))}
            </select>
            <span className="block text-xs text-mid-gray">
              {MODE_OPTIONS.find(
                (option) => option.value === (settings?.data_mode ?? "metadata_only"),
              )?.description ?? ""}
            </span>
          </label>

          <label className="space-y-2 text-sm">
            <span className="block text-xs uppercase tracking-wide text-mid-gray">
              Buffer size
            </span>
            <input
              className="w-full rounded-md border border-mid-gray/20 bg-background px-3 py-2 text-sm"
              type="number"
              min={50}
              max={5000}
              value={settings?.max_events ?? 400}
              onChange={(event) => {
                if (!settings) return;
                const nextValue = Number(event.target.value);
                setSettings({
                  ...settings,
                  max_events: Number.isFinite(nextValue) ? nextValue : settings.max_events,
                });
              }}
              onBlur={() => {
                if (!settings) return;
                void persistSettings({
                  ...settings,
                  max_events: Math.max(50, Math.min(5000, settings.max_events)),
                });
              }}
            />
          </label>

          <label className="flex items-center justify-between gap-3 rounded-md border border-mid-gray/20 bg-background px-3 py-2 text-sm">
            <span>Enabled</span>
            <input
              type="checkbox"
              checked={settings?.enabled ?? true}
              onChange={(event) => {
                if (!settings) return;
                void persistSettings({
                  ...settings,
                  enabled: event.target.checked,
                });
              }}
            />
          </label>

          <label className="flex items-center justify-between gap-3 rounded-md border border-mid-gray/20 bg-background px-3 py-2 text-sm">
            <span>Live stream</span>
            <input
              type="checkbox"
              checked={settings?.emit_live_events ?? true}
              onChange={(event) => {
                if (!settings) return;
                void persistSettings({
                  ...settings,
                  emit_live_events: event.target.checked,
                });
              }}
            />
          </label>
        </div>

        <div className="mt-4 grid gap-3 lg:grid-cols-[1fr_220px_160px_160px]">
          <input
            className="w-full rounded-md border border-mid-gray/20 bg-background px-3 py-2 text-sm"
            placeholder="Search events, status, traces, attributes"
            value={search}
            onChange={(event) => setSearch(event.target.value)}
          />
          <select
            className="rounded-md border border-mid-gray/20 bg-background px-3 py-2 text-sm"
            value={selectedComponent}
            onChange={(event) => setSelectedComponent(event.target.value)}
          >
            {componentOptions.map((component) => (
              <option key={component} value={component}>
                {component === "all" ? "All components" : component}
              </option>
            ))}
          </select>
          <select
            className="rounded-md border border-mid-gray/20 bg-background px-3 py-2 text-sm"
            value={selectedLevel}
            onChange={(event) =>
              setSelectedLevel(event.target.value as "all" | TelemetryLevel)
            }
          >
            <option value="all">All levels</option>
            <option value="trace">Trace</option>
            <option value="debug">Debug</option>
            <option value="info">Info</option>
            <option value="warn">Warn</option>
            <option value="error">Error</option>
          </select>
          <Button
            variant={livePaused ? "secondary" : "ghost"}
            size="sm"
            onClick={() => setLivePaused((value) => !value)}
          >
            {livePaused ? "Resume live" : "Pause live"}
          </Button>
        </div>
      </div>

      <div className="grid gap-4 xl:grid-cols-[1.4fr_2fr]">
        <div className="rounded-lg border border-mid-gray/20 bg-background/40 p-4">
          <h4 className="text-xs font-medium uppercase tracking-wide text-mid-gray">
            Components
          </h4>
          <div className="mt-3 space-y-2">
            {(stats?.component_breakdown ?? []).slice(0, 12).map((component) => (
              <div
                key={component.component}
                className="rounded-md border border-mid-gray/20 bg-background/30 px-3 py-2"
              >
                <div className="flex items-center justify-between gap-3">
                  <div className="min-w-0">
                    <div className="truncate text-sm font-medium">
                      {component.component}
                    </div>
                    <div className="text-xs text-mid-gray">
                      Last event {formatTimestamp(component.last_event_at_ms)}
                    </div>
                  </div>
                  <div className="text-right text-xs text-mid-gray">
                    <div>{component.total_events} total</div>
                    <div>
                      {component.error_events} errors, {component.warn_events} warnings
                    </div>
                  </div>
                </div>
              </div>
            ))}
            {!stats?.component_breakdown?.length && !isLoading && (
              <div className="rounded-md border border-dashed border-mid-gray/20 px-3 py-6 text-center text-sm text-mid-gray">
                No telemetry captured yet.
              </div>
            )}
          </div>
        </div>

        <div className="rounded-lg border border-mid-gray/20 bg-background/40 p-4">
          <div className="flex items-center justify-between gap-3">
            <h4 className="text-xs font-medium uppercase tracking-wide text-mid-gray">
              Event stream
            </h4>
            <span className="text-xs text-mid-gray">
              {filteredEvents.length} visible / {events.length} buffered
            </span>
          </div>

          <div className="mt-3 max-h-[640px] space-y-3 overflow-y-auto pr-1">
            {filteredEvents.map((event) => (
              <div
                key={event.id}
                className="rounded-lg border border-mid-gray/20 bg-background/30 p-3"
              >
                <div className="flex flex-wrap items-center gap-2">
                  <span className="text-xs text-mid-gray">
                    {formatTimestamp(event.timestamp_ms)}
                  </span>
                  <span
                    className={`rounded-full border px-2 py-0.5 text-[10px] uppercase tracking-wide ${levelClasses[event.level]}`}
                  >
                    {event.level}
                  </span>
                  <span className="rounded-full border border-mid-gray/20 px-2 py-0.5 text-[10px] uppercase tracking-wide text-mid-gray">
                    {event.source}
                  </span>
                  {event.status && (
                    <span className="rounded-full border border-mid-gray/20 px-2 py-0.5 text-[10px] text-mid-gray">
                      {event.status}
                    </span>
                  )}
                  {event.duration_ms !== null &&
                    event.duration_ms !== undefined && (
                      <span className="rounded-full border border-mid-gray/20 px-2 py-0.5 text-[10px] text-mid-gray">
                        {event.duration_ms} ms
                      </span>
                    )}
                </div>

                <div className="mt-2 flex flex-wrap items-center gap-2">
                  <span className="font-mono text-xs text-sky-300">
                    {event.component}
                  </span>
                  <span className="text-xs text-mid-gray">/</span>
                  <span className="font-mono text-xs text-text">{event.action}</span>
                </div>

                <p className="mt-2 text-sm text-text">{event.message}</p>

                {(event.trace_id || event.session_id) && (
                  <div className="mt-2 flex flex-wrap gap-2 text-[11px] text-mid-gray">
                    {event.trace_id && <span>trace: {event.trace_id}</span>}
                    {event.session_id && <span>session: {event.session_id}</span>}
                  </div>
                )}

                {!!event.attributes.length && (
                  <div className="mt-3 grid gap-2 md:grid-cols-2">
                    {event.attributes.map((attribute) => (
                      <div
                        key={`${event.id}-${attribute.key}`}
                        className="rounded-md border border-mid-gray/20 bg-background/40 px-2 py-1.5"
                      >
                        <div className="text-[10px] uppercase tracking-wide text-mid-gray">
                          {attribute.key}
                        </div>
                        <div
                          className={`mt-1 break-words text-xs ${
                            attribute.redacted ? "text-amber-300" : "text-text"
                          }`}
                        >
                          {attribute.value}
                        </div>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            ))}

            {!filteredEvents.length && !isLoading && (
              <div className="rounded-md border border-dashed border-mid-gray/20 px-3 py-10 text-center text-sm text-mid-gray">
                No events match the current filters.
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
};
