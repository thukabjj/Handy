import React, { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { type as osType } from "@tauri-apps/plugin-os";
import { useTranslation } from "react-i18next";
import {
  checkScreenRecordingPermission,
  requestScreenRecordingPermission,
} from "tauri-plugin-macos-permissions-api";
import { Button } from "@/components/ui/Button";
import {
  Dropdown,
  SettingContainer,
  Textarea,
  ToggleSwitch,
} from "@/components/ui";
import { Input } from "@/components/ui/Input";

type LlmProvider =
  | "local_open_ai"
  | "open_router"
  | "open_ai"
  | "groq"
  | "together"
  | "fireworks"
  | "custom"
  | "ollama";

interface ScreenVisionSettingsDto {
  enabled: boolean;
  interval_seconds: number;
  duration_seconds: number;
  max_snapshots: number;
  llm_provider: LlmProvider;
  llm_api_key?: string | null;
  llm_base_url?: string | null;
  llm_model: string;
  prompt: string;
}

interface ScreenVisionSnapshotResult {
  timestamp_ms: number;
  capture_path?: string | null;
  analysis: string;
}

interface ScreenVisionStatus {
  running: boolean;
  session_id?: string | null;
  started_at_ms?: number | null;
  expires_at_ms?: number | null;
  snapshots_processed: number;
  last_result?: ScreenVisionSnapshotResult | null;
  last_error?: string | null;
}

const DEFAULTS: ScreenVisionSettingsDto = {
  enabled: false,
  interval_seconds: 5,
  duration_seconds: 30,
  max_snapshots: 6,
  llm_provider: "open_router",
  llm_api_key: "",
  llm_base_url: "",
  llm_model: "qwen/qwen2.5-vl-72b-instruct:free",
  prompt:
    "Analyze this screen for actionable context. Return concise bullet points for what is visible, what requires attention, and recommended next action.",
};

export const ScreenVisionSettings: React.FC = () => {
  const { t } = useTranslation();
  const [settings, setSettings] = useState<ScreenVisionSettingsDto>(DEFAULTS);
  const [status, setStatus] = useState<ScreenVisionStatus | null>(null);
  const [isBusy, setIsBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [hasScreenPermission, setHasScreenPermission] = useState<boolean>(true);
  const isMacOS = osType() === "macos";

  const load = async () => {
    try {
      const [loadedSettings, loadedStatus] = await Promise.all([
        invoke<ScreenVisionSettingsDto>("get_screen_vision_settings"),
        invoke<ScreenVisionStatus>("get_screen_vision_status"),
      ]);
      setSettings({
        ...loadedSettings,
        llm_api_key: loadedSettings.llm_api_key ?? "",
        llm_base_url: loadedSettings.llm_base_url ?? "",
      });
      setStatus(loadedStatus);
      setError(null);
    } catch (e) {
      setError(String(e));
    }
  };

  useEffect(() => {
    load();
  }, []);

  useEffect(() => {
    if (!isMacOS) {
      setHasScreenPermission(true);
      return;
    }
    checkScreenRecordingPermission()
      .then(setHasScreenPermission)
      .catch(() => setHasScreenPermission(false));
  }, [isMacOS]);

  useEffect(() => {
    if (!status?.running) return;
    const id = window.setInterval(() => {
      invoke<ScreenVisionStatus>("get_screen_vision_status")
        .then((s) => setStatus(s))
        .catch(() => null);
    }, 1500);
    return () => window.clearInterval(id);
  }, [status?.running]);

  const saveSettings = async (next = settings) => {
    setIsBusy(true);
    try {
      await invoke("update_screen_vision_settings", {
        settings: {
          ...next,
          llm_api_key: next.llm_api_key?.trim() || null,
          llm_base_url: next.llm_base_url?.trim() || null,
        },
      });
      setError(null);
      await load();
    } catch (e) {
      setError(String(e));
    } finally {
      setIsBusy(false);
    }
  };

  const runSingleTest = async () => {
    setIsBusy(true);
    try {
      const result = await invoke<ScreenVisionSnapshotResult>(
        "test_screen_vision_once",
      );
      setStatus((prev) => ({
        running: prev?.running ?? false,
        session_id: prev?.session_id ?? null,
        started_at_ms: prev?.started_at_ms ?? null,
        expires_at_ms: prev?.expires_at_ms ?? null,
        snapshots_processed: prev?.snapshots_processed ?? 0,
        last_result: result,
        last_error: null,
      }));
      setError(null);
    } catch (e) {
      setError(String(e));
    } finally {
      setIsBusy(false);
    }
  };

  const start = async () => {
    setIsBusy(true);
    try {
      await saveSettings();
      await invoke("start_screen_vision_session");
      const current = await invoke<ScreenVisionStatus>(
        "get_screen_vision_status",
      );
      setStatus(current);
      setError(null);
    } catch (e) {
      setError(String(e));
    } finally {
      setIsBusy(false);
    }
  };

  const stop = async () => {
    setIsBusy(true);
    try {
      await invoke("stop_screen_vision_session");
      const current = await invoke<ScreenVisionStatus>(
        "get_screen_vision_status",
      );
      setStatus(current);
      setError(null);
    } catch (e) {
      setError(String(e));
    } finally {
      setIsBusy(false);
    }
  };

  return (
    <div className="space-y-3">
      <ToggleSwitch
        label={t(
          "settings.activeListening.screenVision.enable",
          "Enable Screen Vision",
        )}
        description={t(
          "settings.activeListening.screenVision.enableDescription",
          "Allow Handy to periodically capture your screen and infer context for guidance.",
        )}
        checked={settings.enabled}
        onChange={(value) => {
          const next = { ...settings, enabled: value };
          setSettings(next);
          void saveSettings(next);
        }}
        grouped={true}
      />

      {isMacOS && !hasScreenPermission && (
        <div className="rounded-md border border-yellow-500/40 bg-yellow-500/10 p-3 text-xs text-yellow-200">
          <p className="mb-2">
            {t(
              "settings.activeListening.screenVision.permissionRequired",
              "Screen Recording permission is required on macOS before Screen Vision can capture snapshots.",
            )}
          </p>
          <div className="flex gap-2">
            <Button
              variant="secondary"
              size="sm"
              onClick={async () => {
                try {
                  await requestScreenRecordingPermission();
                } finally {
                  const granted = await checkScreenRecordingPermission().catch(
                    () => false,
                  );
                  setHasScreenPermission(granted);
                }
              }}
            >
              {t(
                "settings.activeListening.screenVision.requestPermission",
                "Request permission",
              )}
            </Button>
            <Button
              variant="secondary"
              size="sm"
              onClick={() => invoke("open_screen_recording_settings")}
            >
              {t(
                "settings.activeListening.screenVision.openSettings",
                "Open Screen Recording settings",
              )}
            </Button>
          </div>
        </div>
      )}

      {isMacOS ? (
        <SettingContainer
          title={t(
            "settings.activeListening.screenVision.provider",
            "Vision Provider",
          )}
          layout="horizontal"
          grouped={true}
        >
          <div className="min-w-[220px] rounded-md border border-mid-gray/30 bg-mid-gray/10 px-3 py-2 text-sm text-text">
            OpenRouter
          </div>
        </SettingContainer>
      ) : (
        <SettingContainer
          title={t(
            "settings.activeListening.screenVision.provider",
            "Vision Provider",
          )}
          layout="horizontal"
          grouped={true}
        >
          <Dropdown
            selectedValue={settings.llm_provider}
            options={[
              { value: "open_router", label: "OpenRouter" },
              { value: "open_ai", label: "OpenAI" },
              { value: "groq", label: "Groq" },
              { value: "together", label: "Together" },
              { value: "fireworks", label: "Fireworks" },
              { value: "local_open_ai", label: "Local OpenAI-compatible" },
              { value: "custom", label: "Custom" },
              { value: "ollama", label: "Ollama (/v1)" },
            ]}
            onSelect={(value) =>
              setSettings((prev) => ({
                ...prev,
                llm_provider: (value as LlmProvider) ?? prev.llm_provider,
              }))
            }
            className="min-w-[220px]"
          />
        </SettingContainer>
      )}

      <SettingContainer
        title={t("settings.activeListening.screenVision.apiKey", "API Key")}
        layout="horizontal"
        grouped={true}
      >
        <Input
          type="password"
          value={settings.llm_api_key ?? ""}
          onChange={(e) =>
            setSettings((prev) => ({ ...prev, llm_api_key: e.target.value }))
          }
          placeholder="sk-..."
          className="min-w-[280px]"
        />
      </SettingContainer>

      <SettingContainer
        title={t("settings.activeListening.screenVision.baseUrl", "Base URL")}
        layout="horizontal"
        grouped={true}
      >
        <Input
          value={settings.llm_base_url ?? ""}
          onChange={(e) =>
            setSettings((prev) => ({ ...prev, llm_base_url: e.target.value }))
          }
          placeholder="https://openrouter.ai/api/v1"
          className="min-w-[320px]"
        />
      </SettingContainer>

      <SettingContainer
        title={t("settings.activeListening.screenVision.model", "Vision Model")}
        layout="horizontal"
        grouped={true}
      >
        <Input
          value={settings.llm_model}
          onChange={(e) =>
            setSettings((prev) => ({ ...prev, llm_model: e.target.value }))
          }
          placeholder="qwen/qwen2.5-vl-72b-instruct:free"
          className="min-w-[320px]"
        />
      </SettingContainer>

      <div className="grid grid-cols-3 gap-3">
        <SettingContainer
          title={t(
            "settings.activeListening.screenVision.interval",
            "Interval (s)",
          )}
          grouped={true}
        >
          <Input
            type="number"
            min={1}
            max={60}
            value={String(settings.interval_seconds)}
            onChange={(e) =>
              setSettings((prev) => ({
                ...prev,
                interval_seconds: Math.max(1, Number(e.target.value) || 1),
              }))
            }
          />
        </SettingContainer>
        <SettingContainer
          title={t(
            "settings.activeListening.screenVision.duration",
            "Duration (s)",
          )}
          grouped={true}
        >
          <Input
            type="number"
            min={5}
            max={300}
            value={String(settings.duration_seconds)}
            onChange={(e) =>
              setSettings((prev) => ({
                ...prev,
                duration_seconds: Math.max(5, Number(e.target.value) || 5),
              }))
            }
          />
        </SettingContainer>
        <SettingContainer
          title={t(
            "settings.activeListening.screenVision.maxSnapshots",
            "Max Snapshots",
          )}
          grouped={true}
        >
          <Input
            type="number"
            min={1}
            max={30}
            value={String(settings.max_snapshots)}
            onChange={(e) =>
              setSettings((prev) => ({
                ...prev,
                max_snapshots: Math.max(1, Number(e.target.value) || 1),
              }))
            }
          />
        </SettingContainer>
      </div>

      <SettingContainer
        title={t(
          "settings.activeListening.screenVision.prompt",
          "Vision Prompt",
        )}
        grouped={true}
      >
        <Textarea
          value={settings.prompt}
          onChange={(e) =>
            setSettings((prev) => ({ ...prev, prompt: e.target.value }))
          }
          rows={4}
        />
      </SettingContainer>

      <div className="flex gap-2">
        <Button
          variant="secondary"
          size="md"
          onClick={() => saveSettings()}
          disabled={isBusy}
        >
          {t("common.save", "Save")}
        </Button>
        <Button
          variant="secondary"
          size="md"
          onClick={runSingleTest}
          disabled={isBusy || (isMacOS && !hasScreenPermission)}
        >
          {t("settings.activeListening.screenVision.testOnce", "Test once")}
        </Button>
        {!status?.running ? (
          <Button
            variant="primary"
            size="md"
            onClick={start}
            disabled={
              isBusy || !settings.enabled || (isMacOS && !hasScreenPermission)
            }
          >
            {t(
              "settings.activeListening.screenVision.start",
              "Start timed session",
            )}
          </Button>
        ) : (
          <Button
            variant="secondary"
            size="md"
            onClick={stop}
            disabled={isBusy}
          >
            {t("settings.activeListening.screenVision.stop", "Stop")}
          </Button>
        )}
      </div>

      {status && (
        <div className="text-xs text-mid-gray">
          {t("settings.activeListening.screenVision.status", "Status")}:{" "}
          {status.running ? "running" : "idle"} |{" "}
          {t("settings.activeListening.screenVision.snapshots", "snapshots")}:{" "}
          {status.snapshots_processed}
        </div>
      )}

      {status?.last_result?.analysis && (
        <SettingContainer
          title={t(
            "settings.activeListening.screenVision.latestAnalysis",
            "Latest Analysis",
          )}
          grouped={true}
        >
          <Textarea value={status.last_result.analysis} readOnly rows={8} />
        </SettingContainer>
      )}

      {(error || status?.last_error) && (
        <div className="rounded-md border border-red-500/40 bg-red-500/10 p-2 text-xs text-red-200">
          {error || status?.last_error}
        </div>
      )}
    </div>
  );
};
