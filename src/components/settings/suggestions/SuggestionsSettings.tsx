import React, { useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { Trash2, Plus } from "lucide-react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import {
  commands,
  type QuickResponse,
  type SuggestionsSettings as SuggestionsConfig,
} from "@/bindings";
import { SettingsGroup, ToggleSwitch, Slider } from "@/components/ui";
import { SettingContainer } from "@/components/ui/SettingContainer";
import { Button } from "@/components/ui/Button";
import { Input } from "@/components/ui/Input";
import { Textarea } from "@/components/ui/Textarea";

type LiveSuggestion =
  | {
      type: "quick_response";
      text?: string;
      confidence?: number;
      category?: string;
    }
  | { type: "data_point"; fact?: string; source?: string; relevance?: number }
  | {
      type: "talking_point";
      point?: string;
      rationale?: string;
      confidence?: number;
    }
  | { type: "warning"; message?: string; severity?: string };

interface SuggestionsEventPayload {
  session_id: string;
  timestamp: number;
  suggestions: LiveSuggestion[];
}

const defaultSuggestions: Required<
  Pick<
    SuggestionsConfig,
    | "enabled"
    | "rag_suggestions_enabled"
    | "llm_suggestions_enabled"
    | "max_suggestions"
    | "min_confidence"
    | "auto_dismiss_on_copy"
    | "display_duration_seconds"
    | "quick_responses"
  >
> = {
  enabled: false,
  rag_suggestions_enabled: true,
  llm_suggestions_enabled: true,
  max_suggestions: 3,
  min_confidence: 0.5,
  auto_dismiss_on_copy: true,
  display_duration_seconds: 0,
  quick_responses: [],
};

export const SuggestionsSettings: React.FC = () => {
  const { t } = useTranslation();
  const [settings, setSettings] = useState(defaultSuggestions);
  const [loading, setLoading] = useState(true);
  const [name, setName] = useState("");
  const [category, setCategory] = useState("");
  const [triggers, setTriggers] = useState("");
  const [responseTemplate, setResponseTemplate] = useState("");
  const [liveEvents, setLiveEvents] = useState<SuggestionsEventPayload[]>([]);

  const quickResponses = useMemo(
    () => settings.quick_responses ?? [],
    [settings.quick_responses],
  );

  const refresh = async () => {
    setLoading(true);
    try {
      const result = await commands.getSuggestionsSettings();
      if (result.status === "ok") {
        setSettings({
          enabled: result.data.enabled ?? defaultSuggestions.enabled,
          rag_suggestions_enabled:
            result.data.rag_suggestions_enabled ??
            defaultSuggestions.rag_suggestions_enabled,
          llm_suggestions_enabled:
            result.data.llm_suggestions_enabled ??
            defaultSuggestions.llm_suggestions_enabled,
          max_suggestions:
            result.data.max_suggestions ?? defaultSuggestions.max_suggestions,
          min_confidence:
            result.data.min_confidence ?? defaultSuggestions.min_confidence,
          auto_dismiss_on_copy:
            result.data.auto_dismiss_on_copy ??
            defaultSuggestions.auto_dismiss_on_copy,
          display_duration_seconds:
            result.data.display_duration_seconds ??
            defaultSuggestions.display_duration_seconds,
          quick_responses: result.data.quick_responses ?? [],
        });
      }
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    refresh();
  }, []);

  useEffect(() => {
    let unlisten: UnlistenFn | null = null;
    const setup = async () => {
      unlisten = await listen<SuggestionsEventPayload>(
        "suggestions",
        (event) => {
          setLiveEvents((prev) => [event.payload, ...prev].slice(0, 20));
        },
      );
    };
    setup();

    return () => {
      if (unlisten) {
        unlisten();
      }
    };
  }, []);

  const setEnabled = async (enabled: boolean) => {
    await commands.changeSuggestionsEnabledSetting(enabled);
    setSettings((prev) => ({ ...prev, enabled }));
  };

  const setRagEnabled = async (enabled: boolean) => {
    await commands.changeRagSuggestionsEnabled(enabled);
    setSettings((prev) => ({ ...prev, rag_suggestions_enabled: enabled }));
  };

  const setLlmEnabled = async (enabled: boolean) => {
    await commands.changeLlmSuggestionsEnabled(enabled);
    setSettings((prev) => ({ ...prev, llm_suggestions_enabled: enabled }));
  };

  const setMinConfidence = async (value: number) => {
    await commands.changeMinConfidence(value);
    setSettings((prev) => ({ ...prev, min_confidence: value }));
  };

  const setMaxSuggestions = async (value: number) => {
    const intValue = Math.round(value);
    await commands.changeMaxSuggestions(intValue);
    setSettings((prev) => ({ ...prev, max_suggestions: intValue }));
  };

  const setDisplayDuration = async (value: number) => {
    const intValue = Math.round(value);
    await commands.changeDisplayDuration(intValue);
    setSettings((prev) => ({ ...prev, display_duration_seconds: intValue }));
  };

  const setAutoDismiss = async (enabled: boolean) => {
    await commands.changeAutoDismissOnCopy(enabled);
    setSettings((prev) => ({ ...prev, auto_dismiss_on_copy: enabled }));
  };

  const addQuickResponse = async () => {
    if (!name.trim() || !responseTemplate.trim()) return;
    const triggerPhrases = triggers
      .split(",")
      .map((v) => v.trim())
      .filter(Boolean);
    if (triggerPhrases.length === 0) return;

    const item: QuickResponse = {
      id: `qr_${Date.now()}`,
      name: name.trim(),
      category: category.trim() || "general",
      trigger_phrases: triggerPhrases,
      response_template: responseTemplate.trim(),
      enabled: true,
      created_at: Date.now(),
    };

    await commands.addQuickResponse(item);
    setName("");
    setCategory("");
    setTriggers("");
    setResponseTemplate("");
    await refresh();
  };

  const toggleQuickResponse = async (id: string) => {
    await commands.toggleQuickResponse(id);
    await refresh();
  };

  const deleteQuickResponse = async (id: string) => {
    await commands.deleteQuickResponse(id);
    await refresh();
  };

  if (loading) {
    return (
      <div className="max-w-3xl w-full mx-auto space-y-6">
        <SettingsGroup title={t("settings.suggestions.title", "Suggestions")}>
          <div className="p-4 text-sm text-mid-gray">
            {t(
              "settings.suggestions.loading",
              "Loading suggestions settings...",
            )}
          </div>
        </SettingsGroup>
      </div>
    );
  }

  return (
    <div className="max-w-3xl w-full mx-auto space-y-6">
      <SettingsGroup title={t("settings.suggestions.title", "Suggestions")}>
        <ToggleSwitch
          label={t("settings.suggestions.enable.title", "Enable Suggestions")}
          description={t(
            "settings.suggestions.enable.description",
            "Generate real-time conversation suggestions during Active Listening.",
          )}
          checked={settings.enabled}
          onChange={setEnabled}
          grouped={true}
        />
      </SettingsGroup>

      {settings.enabled && (
        <>
          <SettingsGroup title={t("settings.suggestions.behavior", "Behavior")}>
            <ToggleSwitch
              label={t(
                "settings.suggestions.rag.title",
                "Use Knowledge Base (RAG)",
              )}
              description={t(
                "settings.suggestions.rag.description",
                "Include context retrieved from indexed documents.",
              )}
              checked={settings.rag_suggestions_enabled}
              onChange={setRagEnabled}
              grouped={true}
            />
            <ToggleSwitch
              label={t("settings.suggestions.llm.title", "Use LLM Suggestions")}
              description={t(
                "settings.suggestions.llm.description",
                "Generate contextual talking points from the active model.",
              )}
              checked={settings.llm_suggestions_enabled}
              onChange={setLlmEnabled}
              grouped={true}
            />
            <ToggleSwitch
              label={t(
                "settings.suggestions.autoDismiss.title",
                "Auto-dismiss on copy",
              )}
              description={t(
                "settings.suggestions.autoDismiss.description",
                "Hide suggestion cards automatically when copied.",
              )}
              checked={settings.auto_dismiss_on_copy}
              onChange={setAutoDismiss}
              grouped={true}
            />
            <Slider
              label={t("settings.suggestions.max.title", "Maximum suggestions")}
              description={t(
                "settings.suggestions.max.description",
                "Maximum number of suggestions shown at once.",
              )}
              min={1}
              max={10}
              step={1}
              value={settings.max_suggestions}
              onChange={setMaxSuggestions}
              grouped={true}
              formatValue={(v) => `${Math.round(v)}`}
            />
            <Slider
              label={t(
                "settings.suggestions.confidence.title",
                "Minimum confidence",
              )}
              description={t(
                "settings.suggestions.confidence.description",
                "Hide suggestions below this confidence threshold.",
              )}
              min={0}
              max={1}
              step={0.05}
              value={settings.min_confidence}
              onChange={setMinConfidence}
              grouped={true}
              formatValue={(v) => `${Math.round(v * 100)}%`}
            />
            <Slider
              label={t(
                "settings.suggestions.duration.title",
                "Display duration",
              )}
              description={t(
                "settings.suggestions.duration.description",
                "Seconds to keep a suggestion visible (0 = until dismissed).",
              )}
              min={0}
              max={60}
              step={5}
              value={settings.display_duration_seconds}
              onChange={setDisplayDuration}
              grouped={true}
              formatValue={(v) => `${Math.round(v)}s`}
            />
          </SettingsGroup>

          <SettingsGroup
            title={t("settings.suggestions.quickResponses", "Quick Responses")}
          >
            <div className="space-y-3">
              {quickResponses.map((item) => (
                <div
                  key={item.id}
                  className="border border-mid-gray/20 rounded-lg p-3 bg-mid-gray/5"
                >
                  <div className="flex items-start justify-between gap-2">
                    <div className="min-w-0">
                      <p className="text-sm font-medium text-text">
                        {item.name}
                      </p>
                      <p className="text-xs text-mid-gray">{item.category}</p>
                      <p className="text-xs text-mid-gray mt-1">
                        {item.trigger_phrases.join(", ")}
                      </p>
                    </div>
                    <div className="flex items-center gap-2">
                      <label className="flex items-center gap-2 text-xs text-mid-gray">
                        <input
                          type="checkbox"
                          checked={item.enabled ?? true}
                          onChange={() => toggleQuickResponse(item.id)}
                          className="rounded border-mid-gray/40"
                        />
                        {t("common.enabled", "Enabled")}
                      </label>
                      <button
                        className="p-2 rounded hover:bg-red-500/20 text-red-400"
                        onClick={() => deleteQuickResponse(item.id)}
                        aria-label={t("settings.suggestions.delete", "Delete")}
                      >
                        <Trash2 className="h-4 w-4" />
                      </button>
                    </div>
                  </div>
                  <p className="text-sm text-text/85 mt-2">
                    {item.response_template}
                  </p>
                </div>
              ))}
            </div>

            <div className="mt-4 border-t border-mid-gray/20 pt-4">
              <SettingContainer
                title={t(
                  "settings.suggestions.add.title",
                  "Add quick response",
                )}
                layout="stacked"
                grouped={true}
              >
                <div className="space-y-2">
                  <Input
                    value={name}
                    onChange={(e) => setName(e.target.value)}
                    placeholder={t("settings.suggestions.add.name", "Name")}
                  />
                  <Input
                    value={category}
                    onChange={(e) => setCategory(e.target.value)}
                    placeholder={t(
                      "settings.suggestions.add.category",
                      "Category",
                    )}
                  />
                  <Input
                    value={triggers}
                    onChange={(e) => setTriggers(e.target.value)}
                    placeholder={t(
                      "settings.suggestions.add.triggers",
                      "Trigger phrases (comma-separated)",
                    )}
                  />
                  <Textarea
                    value={responseTemplate}
                    onChange={(e) => setResponseTemplate(e.target.value)}
                    rows={3}
                    placeholder={t(
                      "settings.suggestions.add.response",
                      "Suggested response template",
                    )}
                  />
                  <div className="flex justify-end">
                    <Button
                      onClick={addQuickResponse}
                      variant="primary"
                      size="sm"
                    >
                      <Plus className="h-4 w-4 mr-1" />
                      {t("settings.suggestions.add.button", "Add")}
                    </Button>
                  </div>
                </div>
              </SettingContainer>
            </div>
          </SettingsGroup>

          <SettingsGroup
            title={t(
              "settings.suggestions.live.title",
              "Live Suggestions Feed",
            )}
          >
            <div className="flex justify-end">
              <Button
                variant="ghost"
                size="sm"
                onClick={() => setLiveEvents([])}
              >
                {t("settings.suggestions.live.clear", "Clear")}
              </Button>
            </div>
            {liveEvents.length === 0 ? (
              <div className="text-sm text-mid-gray p-3">
                {t(
                  "settings.suggestions.live.empty",
                  "No suggestions received yet. Start Active Listening and speak to generate suggestions.",
                )}
              </div>
            ) : (
              <div className="space-y-3">
                {liveEvents.map((evt, idx) => (
                  <div
                    key={`${evt.session_id}-${evt.timestamp}-${idx}`}
                    className="border border-mid-gray/20 rounded-lg p-3 bg-mid-gray/5"
                  >
                    <div className="flex items-center justify-between gap-2 text-xs text-mid-gray">
                      <span>
                        {t("settings.suggestions.live.session", "Session")}:{" "}
                        {evt.session_id}
                      </span>
                      <span>
                        {new Date(evt.timestamp).toLocaleTimeString()}
                      </span>
                    </div>
                    <div className="mt-2 space-y-2">
                      {evt.suggestions.map((suggestion, suggestionIdx) => (
                        <div
                          key={`${evt.timestamp}-${suggestionIdx}`}
                          className="rounded border border-mid-gray/15 p-2 bg-background/40"
                        >
                          <p className="text-xs uppercase text-mid-gray mb-1">
                            {suggestion.type.replace("_", " ")}
                          </p>
                          {"text" in suggestion && suggestion.text && (
                            <p className="text-sm text-text">
                              {suggestion.text}
                            </p>
                          )}
                          {"point" in suggestion && suggestion.point && (
                            <p className="text-sm text-text">
                              {suggestion.point}
                            </p>
                          )}
                          {"fact" in suggestion && suggestion.fact && (
                            <p className="text-sm text-text">
                              {suggestion.fact}
                            </p>
                          )}
                          {"message" in suggestion && suggestion.message && (
                            <p className="text-sm text-text">
                              {suggestion.message}
                            </p>
                          )}
                        </div>
                      ))}
                    </div>
                  </div>
                ))}
              </div>
            )}
          </SettingsGroup>
        </>
      )}
    </div>
  );
};
