import React, { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { RefreshCcw, Wifi, WifiOff, Presentation } from "lucide-react";
import { commands, type LlmProvider } from "@/bindings";
import { SpeakerAnalytics } from "./SpeakerAnalytics";

import {
  Dropdown,
  SettingContainer,
  SettingsGroup,
  ToggleSwitch,
  Textarea,
} from "@/components/ui";
import { Button } from "../../ui/Button";
import { ResetButton } from "../../ui/ResetButton";
import { Input } from "../../ui/Input";
import { useSettings } from "../../../hooks/useSettings";
import { ShortcutInput as HandyShortcut } from "../ShortcutInput";
import { SessionViewer } from "./SessionViewer";
import { AudioSourceSettings } from "./AudioSourceSettings";
import { WakeWordSettings } from "./WakeWordSettings";
import { ScreenVisionSettings } from "./ScreenVisionSettings";
import {
  CLOUD_PROVIDER_PRESETS,
  type CloudProviderId,
  detectCloudProvider,
  prioritizeOpenRouterModels,
  resolveCloudFetchConfig,
} from "./providerUtils";

const TUCANO2_MODEL_PRESETS = [
  "Polygl0t/Tucano2-qwen-1.5B-Instruct",
  "Polygl0t/Tucano2-qwen-3.7B-Instruct",
  "Polygl0t/Tucano2-qwen-6.7B-Instruct",
];

const DisabledNotice: React.FC<{ children: React.ReactNode }> = ({
  children,
}) => (
  <div className="p-4 bg-mid-gray/5 rounded-lg border border-mid-gray/20">
    <p className="text-sm text-mid-gray">{children}</p>
  </div>
);

const OllamaConnectionStatus: React.FC = () => {
  const { t } = useTranslation();
  const [isConnected, setIsConnected] = useState<boolean | null>(null);
  const [isChecking, setIsChecking] = useState(false);

  const checkConnection = async () => {
    setIsChecking(true);
    try {
      const result = await commands.checkOllamaConnection();
      if (result.status === "ok") {
        setIsConnected(result.data);
      } else {
        setIsConnected(false);
      }
    } catch {
      setIsConnected(false);
    } finally {
      setIsChecking(false);
    }
  };

  useEffect(() => {
    checkConnection();
  }, []);

  const statusText = isChecking
    ? t("settings.activeListening.ollama.connectionStatus.checking")
    : isConnected
      ? t("settings.activeListening.ollama.connectionStatus.connected")
      : t("settings.activeListening.ollama.connectionStatus.disconnected");

  const StatusIcon = isConnected ? Wifi : WifiOff;
  const statusColor = isConnected ? "text-green-500" : "text-red-500";

  return (
    <div className="flex items-center gap-2 p-3 bg-mid-gray/5 rounded-lg border border-mid-gray/20">
      <StatusIcon className={`h-4 w-4 ${statusColor}`} />
      <span className={`text-sm ${statusColor}`}>{statusText}</span>
      <button
        onClick={checkConnection}
        disabled={isChecking}
        className="ml-auto p-1 hover:bg-mid-gray/20 rounded transition-colors"
        title={t("settings.activeListening.ollama.refreshConnection")}
        aria-label={t("settings.activeListening.ollama.refreshConnection")}
      >
        <RefreshCcw className={`h-4 w-4 ${isChecking ? "animate-spin" : ""}`} />
      </button>
    </div>
  );
};

const LlmProviderSettings: React.FC = () => {
  const { t } = useTranslation();
  const { getSetting, refreshSettings } = useSettings();
  const [modelOptions, setModelOptions] = useState<string[]>([]);
  const [isFetchingModels, setIsFetchingModels] = useState(false);

  const activeListening = getSetting("active_listening");
  const provider: LlmProvider = activeListening?.llm_provider ?? "ollama";
  const baseUrl = activeListening?.ollama_base_url ?? "http://localhost:11434";
  const ollamaModel = activeListening?.ollama_model ?? "";
  const llmModel = activeListening?.llm_model ?? "";
  const llmApiKey = activeListening?.llm_api_key ?? "";
  const llmBaseUrl = activeListening?.llm_base_url ?? "";
  const isLocalProvider = provider === "ollama" || provider === "local_open_ai";
  const mode = isLocalProvider ? "local" : "cloud";
  const cloudProvider = detectCloudProvider(provider, llmBaseUrl);
  const [ollamaBaseUrlDraft, setOllamaBaseUrlDraft] = useState(baseUrl);
  const [llmApiKeyDraft, setLlmApiKeyDraft] = useState(llmApiKey);
  const [llmBaseUrlDraft, setLlmBaseUrlDraft] = useState(llmBaseUrl);
  const [llmModelDraft, setLlmModelDraft] = useState(llmModel);

  useEffect(() => {
    setOllamaBaseUrlDraft(baseUrl);
  }, [baseUrl]);

  useEffect(() => {
    setLlmApiKeyDraft(llmApiKey);
  }, [llmApiKey]);

  useEffect(() => {
    setLlmBaseUrlDraft(llmBaseUrl);
  }, [llmBaseUrl]);

  useEffect(() => {
    setLlmModelDraft(llmModel);
  }, [llmModel]);

  const fetchOllamaModels = async () => {
    setIsFetchingModels(true);
    try {
      const result = await commands.fetchOllamaModels();
      if (result.status === "ok") {
        setModelOptions(result.data.map((m) => m.name));
      }
    } catch (error) {
      console.error("Failed to fetch Ollama models:", error);
    } finally {
      setIsFetchingModels(false);
    }
  };

  const fetchCloudModels = async () => {
    if (!llmApiKeyDraft.trim()) return;

    const { provider: effectiveProvider, baseUrl: effectiveBaseUrl } =
      resolveCloudFetchConfig(cloudProvider, llmBaseUrlDraft);

    setIsFetchingModels(true);
    try {
      const result = await commands.fetchLlmModels(
        effectiveProvider,
        llmApiKeyDraft,
        effectiveBaseUrl,
      );
      if (result.status === "ok") {
        const models =
          cloudProvider === "open_router"
            ? prioritizeOpenRouterModels(result.data)
            : result.data;
        setModelOptions(models);
      }
    } catch (error) {
      console.error("Failed to fetch LLM models:", error);
    } finally {
      setIsFetchingModels(false);
    }
  };

  const fetchLocalOpenAiModels = async () => {
    setIsFetchingModels(true);
    try {
      const result = await commands.fetchLlmModels(
        "local_open_ai",
        llmApiKeyDraft,
        llmBaseUrlDraft || "http://127.0.0.1:8000/v1",
      );
      if (result.status === "ok") {
        setModelOptions(result.data);
      }
    } catch (error) {
      console.error("Failed to fetch local OpenAI-compatible models:", error);
    } finally {
      setIsFetchingModels(false);
    }
  };

  useEffect(() => {
    setModelOptions([]);
    if (provider === "ollama") {
      fetchOllamaModels();
    }
  }, [provider, baseUrl]);

  const handleModeChange = async (value: string | null) => {
    if (!value) return;
    if (value === "local") {
      await commands.changeActiveListeningLlmProvider("ollama");
      await refreshSettings();
      return;
    }

    if (isLocalProvider) {
      await commands.changeActiveListeningLlmProvider("open_router");
      await refreshSettings();
    }
  };

  const handleLocalRuntimeChange = async (value: string | null) => {
    if (!value) return;
    if (value === "ollama") {
      await commands.changeActiveListeningLlmProvider("ollama");
    } else if (value === "local_open_ai") {
      await commands.changeActiveListeningLlmProvider("local_open_ai");
      if (!llmBaseUrlDraft.trim()) {
        await commands.changeActiveListeningLlmBaseUrl(
          "http://127.0.0.1:8000/v1",
        );
      }
    }
    await refreshSettings();
  };

  const handleCloudProviderChange = async (value: string | null) => {
    if (!value) return;
    const selected = value as CloudProviderId;

    if (selected === "open_router") {
      await commands.changeActiveListeningLlmProvider("open_router");
      await refreshSettings();
      return;
    }

    if (selected === "custom") {
      await commands.changeActiveListeningLlmProvider("custom");
      await refreshSettings();
      return;
    }

    await commands.changeActiveListeningLlmProvider(selected as LlmProvider);
    await refreshSettings();
  };

  const handleBaseUrlChange = async (newUrl: string) => {
    await commands.changeOllamaBaseUrlSetting(newUrl);
    await refreshSettings();
    fetchOllamaModels();
  };

  const handleOllamaModelChange = async (newModel: string | null) => {
    if (!newModel) return;
    await commands.changeOllamaModelSetting(newModel);
    await refreshSettings();
  };

  const handleApiKeyChange = async (key: string) => {
    await commands.changeActiveListeningLlmApiKey(key);
    await refreshSettings();
  };

  const handleLlmModelChange = async (newModel: string | null) => {
    if (!newModel) return;
    await commands.changeActiveListeningLlmModel(newModel);
    await refreshSettings();
  };

  const handleLlmBaseUrlChange = async (url: string) => {
    await commands.changeActiveListeningLlmBaseUrl(url);
    await refreshSettings();
  };

  return (
    <>
      <SettingContainer
        title={t("settings.activeListening.llm.mode.title")}
        description={t("settings.activeListening.llm.mode.description")}
        descriptionMode="tooltip"
        layout="horizontal"
        grouped={true}
      >
        <Dropdown
          selectedValue={mode}
          options={[
            {
              value: "local",
              label: t("settings.activeListening.llm.mode.local"),
            },
            {
              value: "cloud",
              label: t("settings.activeListening.llm.mode.cloud"),
            },
          ]}
          onSelect={handleModeChange}
          className="min-w-[200px]"
        />
      </SettingContainer>

      {mode === "local" && (
        <>
          <SettingContainer
            title={t(
              "settings.activeListening.llm.runtime.title",
              "Local Runtime",
            )}
            description={t(
              "settings.activeListening.llm.runtime.description",
              "Choose local inference runtime",
            )}
            descriptionMode="tooltip"
            layout="horizontal"
            grouped={true}
          >
            <Dropdown
              selectedValue={
                provider === "local_open_ai" ? "local_open_ai" : "ollama"
              }
              options={[
                {
                  value: "ollama",
                  label: t(
                    "settings.activeListening.llm.runtime.ollama",
                    "Ollama",
                  ),
                },
                {
                  value: "local_open_ai",
                  label: t(
                    "settings.activeListening.llm.runtime.localOpenAi",
                    "OpenAI-compatible local server (vLLM/TGI)",
                  ),
                },
              ]}
              onSelect={handleLocalRuntimeChange}
              className="min-w-[300px]"
            />
          </SettingContainer>

          {provider === "ollama" && (
            <>
              <OllamaConnectionStatus />
              <SettingContainer
                title={t("settings.activeListening.ollama.baseUrl.title")}
                description={t(
                  "settings.activeListening.ollama.baseUrl.description",
                )}
                descriptionMode="tooltip"
                layout="horizontal"
                grouped={true}
              >
                <Input
                  value={ollamaBaseUrlDraft}
                  onChange={(e) => setOllamaBaseUrlDraft(e.target.value)}
                  onBlur={() => handleBaseUrlChange(ollamaBaseUrlDraft)}
                  placeholder="http://localhost:11434"
                  className="min-w-[300px]"
                />
              </SettingContainer>

              <SettingContainer
                title={t("settings.activeListening.ollama.model.title")}
                description={t(
                  "settings.activeListening.ollama.model.description",
                )}
                descriptionMode="tooltip"
                layout="horizontal"
                grouped={true}
              >
                <div className="flex items-center gap-2">
                  <Dropdown
                    selectedValue={ollamaModel || null}
                    options={modelOptions.map((m) => ({ value: m, label: m }))}
                    onSelect={handleOllamaModelChange}
                    placeholder={
                      modelOptions.length > 0
                        ? t("settings.activeListening.ollama.model.placeholder")
                        : t(
                            "settings.activeListening.ollama.model.placeholderNoModels",
                          )
                    }
                    disabled={isFetchingModels}
                    className="min-w-[250px]"
                  />
                  <ResetButton
                    onClick={fetchOllamaModels}
                    disabled={isFetchingModels}
                    ariaLabel={t(
                      "settings.activeListening.ollama.model.refresh",
                    )}
                    className="flex h-10 w-10 items-center justify-center"
                  >
                    <RefreshCcw
                      className={`h-4 w-4 ${isFetchingModels ? "animate-spin" : ""}`}
                    />
                  </ResetButton>
                </div>
              </SettingContainer>
            </>
          )}

          {provider === "local_open_ai" && (
            <>
              <SettingContainer
                title={t(
                  "settings.activeListening.llm.model.tucanoPresetsTitle",
                  "Tucano2 Presets",
                )}
                description={t(
                  "settings.activeListening.llm.model.tucanoPresetsDescription",
                  "Quick-select a Tucano2 model served by your local vLLM/TGI runtime",
                )}
                descriptionMode="tooltip"
                layout="horizontal"
                grouped={true}
              >
                <Dropdown
                  selectedValue={
                    TUCANO2_MODEL_PRESETS.includes(llmModel) ? llmModel : null
                  }
                  options={TUCANO2_MODEL_PRESETS.map((m) => ({
                    value: m,
                    label: m,
                  }))}
                  onSelect={handleLlmModelChange}
                  placeholder={t(
                    "settings.activeListening.llm.model.tucanoPresetsPlaceholder",
                    "Select Tucano2 preset...",
                  )}
                  className="min-w-[320px]"
                />
              </SettingContainer>

              <SettingContainer
                title={t("settings.activeListening.llm.baseUrl.title")}
                description={t(
                  "settings.activeListening.llm.baseUrl.description",
                )}
                descriptionMode="tooltip"
                layout="horizontal"
                grouped={true}
              >
                <Input
                  value={llmBaseUrlDraft}
                  onChange={(e) => setLlmBaseUrlDraft(e.target.value)}
                  onBlur={() => handleLlmBaseUrlChange(llmBaseUrlDraft)}
                  placeholder="http://127.0.0.1:8000/v1"
                  className="min-w-[300px]"
                />
              </SettingContainer>
              <SettingContainer
                title={t("settings.activeListening.llm.apiKey.title")}
                description={t(
                  "settings.activeListening.llm.apiKey.descriptionLocal",
                  "Optional API key for local OpenAI-compatible server",
                )}
                descriptionMode="tooltip"
                layout="horizontal"
                grouped={true}
              >
                <Input
                  type="password"
                  value={llmApiKeyDraft}
                  onChange={(e) => setLlmApiKeyDraft(e.target.value)}
                  onBlur={() => handleApiKeyChange(llmApiKeyDraft)}
                  placeholder="optional"
                  className="min-w-[300px]"
                />
              </SettingContainer>
              <SettingContainer
                title={t("settings.activeListening.llm.model.title")}
                description={t(
                  "settings.activeListening.llm.model.description",
                )}
                descriptionMode="tooltip"
                layout="horizontal"
                grouped={true}
              >
                <div className="flex items-center gap-2">
                  {modelOptions.length > 0 ? (
                    <Dropdown
                      selectedValue={llmModel || null}
                      options={modelOptions.map((m) => ({
                        value: m,
                        label: m,
                      }))}
                      onSelect={handleLlmModelChange}
                      placeholder={t(
                        "settings.activeListening.llm.model.placeholder",
                      )}
                      disabled={isFetchingModels}
                      className="min-w-[250px]"
                    />
                  ) : (
                    <Input
                      value={llmModelDraft}
                      onChange={(e) => setLlmModelDraft(e.target.value)}
                      onBlur={() => handleLlmModelChange(llmModelDraft)}
                      placeholder={t(
                        "settings.activeListening.llm.model.inputPlaceholder",
                      )}
                      className="min-w-[250px]"
                    />
                  )}
                  <ResetButton
                    onClick={fetchLocalOpenAiModels}
                    disabled={isFetchingModels}
                    ariaLabel={t(
                      "settings.activeListening.llm.model.refresh",
                      "Refresh models",
                    )}
                    className="flex h-10 w-10 items-center justify-center"
                  >
                    <RefreshCcw
                      className={`h-4 w-4 ${isFetchingModels ? "animate-spin" : ""}`}
                    />
                  </ResetButton>
                </div>
                <p className="mt-2 text-xs text-mid-gray">
                  {t(
                    "settings.activeListening.llm.model.tucanoHint",
                    "Tip: run Tucano2 locally via vLLM/TGI and select it here.",
                  )}
                </p>
              </SettingContainer>
            </>
          )}
        </>
      )}

      {mode === "cloud" && (
        <>
          <SettingContainer
            title={t("settings.activeListening.llm.provider.title")}
            description={t("settings.activeListening.llm.provider.description")}
            descriptionMode="tooltip"
            layout="horizontal"
            grouped={true}
          >
            <Dropdown
              selectedValue={cloudProvider}
              options={[
                {
                  value: "open_router",
                  label: t("settings.activeListening.llm.provider.openRouter"),
                },
                {
                  value: "open_ai",
                  label: t("settings.activeListening.llm.provider.openAi"),
                },
                {
                  value: "groq",
                  label: t("settings.activeListening.llm.provider.groq"),
                },
                {
                  value: "together",
                  label: t("settings.activeListening.llm.provider.together"),
                },
                {
                  value: "fireworks",
                  label: t("settings.activeListening.llm.provider.fireworks"),
                },
                {
                  value: "custom",
                  label: t("settings.activeListening.llm.provider.custom"),
                },
              ]}
              onSelect={handleCloudProviderChange}
              className="min-w-[240px]"
            />
          </SettingContainer>

          {cloudProvider === "custom" && (
            <SettingContainer
              title={t("settings.activeListening.llm.baseUrl.title")}
              description={t(
                "settings.activeListening.llm.baseUrl.description",
              )}
              descriptionMode="tooltip"
              layout="horizontal"
              grouped={true}
            >
              <Input
                value={llmBaseUrlDraft}
                onChange={(e) => setLlmBaseUrlDraft(e.target.value)}
                onBlur={() => handleLlmBaseUrlChange(llmBaseUrlDraft)}
                placeholder="https://api.example.com/v1"
                className="min-w-[300px]"
              />
            </SettingContainer>
          )}

          <SettingContainer
            title={t("settings.activeListening.llm.apiKey.title")}
            description={t("settings.activeListening.llm.apiKey.description")}
            descriptionMode="tooltip"
            layout="horizontal"
            grouped={true}
          >
            <Input
              type="password"
              value={llmApiKeyDraft}
              onChange={(e) => setLlmApiKeyDraft(e.target.value)}
              onBlur={() => handleApiKeyChange(llmApiKeyDraft)}
              placeholder="sk-..."
              className="min-w-[300px]"
            />
          </SettingContainer>

          <SettingContainer
            title={t("settings.activeListening.llm.model.title")}
            description={t("settings.activeListening.llm.model.description")}
            descriptionMode="tooltip"
            layout="horizontal"
            grouped={true}
          >
            <div className="flex items-center gap-2">
              {modelOptions.length > 0 ? (
                <Dropdown
                  selectedValue={llmModel || null}
                  options={modelOptions.map((m) => ({ value: m, label: m }))}
                  onSelect={handleLlmModelChange}
                  placeholder={t(
                    "settings.activeListening.llm.model.placeholder",
                  )}
                  disabled={isFetchingModels}
                  className="min-w-[250px]"
                />
              ) : (
                <Input
                  value={llmModelDraft}
                  onChange={(e) => setLlmModelDraft(e.target.value)}
                  onBlur={() => handleLlmModelChange(llmModelDraft)}
                  placeholder={t(
                    "settings.activeListening.llm.model.inputPlaceholder",
                  )}
                  className="min-w-[250px]"
                />
              )}
              <ResetButton
                onClick={fetchCloudModels}
                disabled={isFetchingModels || !llmApiKeyDraft.trim()}
                ariaLabel={t(
                  "settings.activeListening.llm.model.refresh",
                  "Refresh models",
                )}
                className="flex h-10 w-10 items-center justify-center"
              >
                <RefreshCcw
                  className={`h-4 w-4 ${isFetchingModels ? "animate-spin" : ""}`}
                />
              </ResetButton>
            </div>
            {cloudProvider === "open_router" && (
              <p className="mt-2 text-xs text-mid-gray">
                {t("settings.activeListening.llm.model.openSourceHint")}
              </p>
            )}
          </SettingContainer>
        </>
      )}
    </>
  );
};

const SegmentSettingsComponent: React.FC = () => {
  const { t } = useTranslation();
  const { getSetting, refreshSettings } = useSettings();

  const activeListening = getSetting("active_listening");
  const segmentDuration =
    activeListening?.segment_duration_seconds?.toString() ?? "15";

  const handleDurationChange = async (value: string | null) => {
    if (!value) return;
    await commands.changeActiveListeningSegmentDurationSetting(parseInt(value));
    await refreshSettings();
  };

  return (
    <SettingContainer
      title={t("settings.activeListening.segments.duration.title")}
      description={t("settings.activeListening.segments.duration.description")}
      descriptionMode="tooltip"
      layout="horizontal"
      grouped={true}
    >
      <Dropdown
        selectedValue={segmentDuration}
        options={[
          {
            value: "10",
            label: t("settings.activeListening.segments.duration.seconds", {
              count: 10,
            }),
          },
          {
            value: "15",
            label: t("settings.activeListening.segments.duration.seconds", {
              count: 15,
            }),
          },
          {
            value: "20",
            label: t("settings.activeListening.segments.duration.seconds", {
              count: 20,
            }),
          },
          {
            value: "30",
            label: t("settings.activeListening.segments.duration.seconds", {
              count: 30,
            }),
          },
        ]}
        onSelect={handleDurationChange}
      />
    </SettingContainer>
  );
};

// Helper to group prompts by category
type PromptCategory = "note_taking" | "meeting_coach" | "custom";

const getCategoryLabel = (
  category: PromptCategory,
  t: ReturnType<typeof useTranslation>["t"],
): string => {
  return t(`settings.activeListening.prompts.categories.${category}`);
};

const getCategoryOrder = (category: PromptCategory): number => {
  switch (category) {
    case "meeting_coach":
      return 0; // Meeting Coach first (most relevant for real-time assistance)
    case "note_taking":
      return 1;
    case "custom":
      return 2;
    default:
      return 3;
  }
};

const PromptsEditorComponent: React.FC = () => {
  const { t } = useTranslation();
  const { getSetting, refreshSettings } = useSettings();
  const [isCreating, setIsCreating] = useState(false);
  const [draftName, setDraftName] = useState("");
  const [draftTemplate, setDraftTemplate] = useState("");

  const activeListening = getSetting("active_listening");
  const prompts = activeListening?.prompts ?? [];
  const selectedPromptId = activeListening?.selected_prompt_id ?? "";
  const selectedPrompt = prompts.find((p) => p.id === selectedPromptId) || null;

  // Group prompts by category for the dropdown
  const groupedOptions = React.useMemo(() => {
    const groups: Record<PromptCategory, typeof prompts> = {
      meeting_coach: [],
      note_taking: [],
      custom: [],
    };

    for (const prompt of prompts) {
      // Use 'category' field if available, fallback to heuristics
      const category: PromptCategory =
        (prompt as { category?: PromptCategory }).category ||
        (prompt.is_default
          ? prompt.id.startsWith("meeting_coach")
            ? "meeting_coach"
            : "note_taking"
          : "custom");
      groups[category].push(prompt);
    }

    return Object.entries(groups)
      .filter(([, list]) => list.length > 0)
      .sort(
        ([a], [b]) =>
          getCategoryOrder(a as PromptCategory) -
          getCategoryOrder(b as PromptCategory),
      )
      .map(([category, list]) => ({
        label: getCategoryLabel(category as PromptCategory, t),
        options: list.map((p) => ({
          value: p.id,
          label: p.name,
        })),
      }));
  }, [prompts, t]);

  useEffect(() => {
    if (isCreating) return;

    if (selectedPrompt) {
      setDraftName(selectedPrompt.name);
      setDraftTemplate(selectedPrompt.prompt_template);
    } else {
      setDraftName("");
      setDraftTemplate("");
    }
  }, [
    isCreating,
    selectedPromptId,
    selectedPrompt?.name,
    selectedPrompt?.prompt_template,
  ]);

  const handlePromptSelect = async (promptId: string | null) => {
    if (!promptId) return;
    await commands.setActiveListeningSelectedPrompt(promptId);
    await refreshSettings();
    setIsCreating(false);
  };

  const handleCreatePrompt = async () => {
    if (!draftName.trim() || !draftTemplate.trim()) return;

    try {
      const result = await commands.addActiveListeningPrompt(
        draftName.trim(),
        draftTemplate.trim(),
      );
      if (result.status === "ok") {
        await refreshSettings();
        await commands.setActiveListeningSelectedPrompt(result.data.id);
        await refreshSettings();
        setIsCreating(false);
      }
    } catch (error) {
      console.error("Failed to create prompt:", error);
    }
  };

  const handleUpdatePrompt = async () => {
    if (!selectedPromptId || !draftName.trim() || !draftTemplate.trim()) return;

    try {
      await commands.updateActiveListeningPrompt(
        selectedPromptId,
        draftName.trim(),
        draftTemplate.trim(),
      );
      await refreshSettings();
    } catch (error) {
      console.error("Failed to update prompt:", error);
    }
  };

  const handleDeletePrompt = async (promptId: string) => {
    if (!promptId) return;

    try {
      await commands.deleteActiveListeningPrompt(promptId);
      await refreshSettings();
      setIsCreating(false);
    } catch (error) {
      console.error("Failed to delete prompt:", error);
    }
  };

  const handleCancelCreate = () => {
    setIsCreating(false);
    if (selectedPrompt) {
      setDraftName(selectedPrompt.name);
      setDraftTemplate(selectedPrompt.prompt_template);
    } else {
      setDraftName("");
      setDraftTemplate("");
    }
  };

  const handleStartCreate = () => {
    setIsCreating(true);
    setDraftName("");
    setDraftTemplate("");
  };

  const hasPrompts = prompts.length > 0;
  const isDirty =
    !!selectedPrompt &&
    (draftName.trim() !== selectedPrompt.name ||
      draftTemplate.trim() !== selectedPrompt.prompt_template.trim());

  return (
    <SettingContainer
      title={t("settings.activeListening.prompts.selectedPrompt.title")}
      description={t(
        "settings.activeListening.prompts.selectedPrompt.description",
      )}
      descriptionMode="tooltip"
      layout="stacked"
      grouped={true}
    >
      <div className="space-y-3">
        <div className="flex gap-2">
          <Dropdown
            selectedValue={selectedPromptId || null}
            options={groupedOptions}
            onSelect={(value) => handlePromptSelect(value)}
            placeholder={
              prompts.length === 0
                ? t("settings.activeListening.prompts.noPrompts")
                : t("settings.activeListening.prompts.selectPrompt")
            }
            disabled={isCreating}
            className="flex-1"
          />
          <Button
            onClick={handleStartCreate}
            variant="primary"
            size="md"
            disabled={isCreating}
          >
            {t("settings.activeListening.prompts.createNew")}
          </Button>
        </div>

        {!isCreating && hasPrompts && selectedPrompt && (
          <div className="space-y-3">
            <div className="space-y-2 flex flex-col">
              <label className="text-sm font-semibold">
                {t("settings.activeListening.prompts.promptLabel")}
              </label>
              <Input
                type="text"
                value={draftName}
                onChange={(e) => setDraftName(e.target.value)}
                placeholder={t(
                  "settings.activeListening.prompts.promptLabelPlaceholder",
                )}
                variant="compact"
              />
            </div>

            <div className="space-y-2 flex flex-col">
              <label className="text-sm font-semibold">
                {t("settings.activeListening.prompts.promptTemplate")}
              </label>
              <Textarea
                value={draftTemplate}
                onChange={(e) => setDraftTemplate(e.target.value)}
                placeholder={t(
                  "settings.activeListening.prompts.promptTemplatePlaceholder",
                )}
                rows={6}
              />
              <p
                className="text-xs text-mid-gray/70"
                dangerouslySetInnerHTML={{
                  __html: t("settings.activeListening.prompts.promptTip"),
                }}
              />
            </div>

            <div className="flex gap-2 pt-2">
              <Button
                onClick={handleUpdatePrompt}
                variant="primary"
                size="md"
                disabled={
                  !draftName.trim() || !draftTemplate.trim() || !isDirty
                }
              >
                {t("settings.activeListening.prompts.updatePrompt")}
              </Button>
              <Button
                onClick={() => handleDeletePrompt(selectedPromptId)}
                variant="secondary"
                size="md"
                disabled={
                  !selectedPromptId ||
                  prompts.length <= 1 ||
                  selectedPrompt?.is_default
                }
              >
                {t("settings.activeListening.prompts.deletePrompt")}
              </Button>
            </div>
          </div>
        )}

        {!isCreating && !selectedPrompt && (
          <div className="p-3 bg-mid-gray/5 rounded border border-mid-gray/20">
            <p className="text-sm text-mid-gray">
              {hasPrompts
                ? t("settings.activeListening.prompts.selectToEdit")
                : t("settings.activeListening.prompts.createFirst")}
            </p>
          </div>
        )}

        {isCreating && (
          <div className="space-y-3">
            <div className="space-y-2 block flex flex-col">
              <label className="text-sm font-semibold text-text">
                {t("settings.activeListening.prompts.promptLabel")}
              </label>
              <Input
                type="text"
                value={draftName}
                onChange={(e) => setDraftName(e.target.value)}
                placeholder={t(
                  "settings.activeListening.prompts.promptLabelPlaceholder",
                )}
                variant="compact"
              />
            </div>

            <div className="space-y-2 flex flex-col">
              <label className="text-sm font-semibold">
                {t("settings.activeListening.prompts.promptTemplate")}
              </label>
              <Textarea
                value={draftTemplate}
                onChange={(e) => setDraftTemplate(e.target.value)}
                placeholder={t(
                  "settings.activeListening.prompts.promptTemplatePlaceholder",
                )}
                rows={6}
              />
              <p
                className="text-xs text-mid-gray/70"
                dangerouslySetInnerHTML={{
                  __html: t("settings.activeListening.prompts.promptTip"),
                }}
              />
            </div>

            <div className="flex gap-2 pt-2">
              <Button
                onClick={handleCreatePrompt}
                variant="primary"
                size="md"
                disabled={!draftName.trim() || !draftTemplate.trim()}
              >
                {t("settings.activeListening.prompts.createPrompt")}
              </Button>
              <Button
                onClick={handleCancelCreate}
                variant="secondary"
                size="md"
              >
                {t("settings.activeListening.prompts.cancel")}
              </Button>
            </div>
          </div>
        )}
      </div>
    </SettingContainer>
  );
};

export const ActiveListeningSettings: React.FC = () => {
  const { t } = useTranslation();
  const { getSetting, refreshSettings } = useSettings();

  const activeListening = getSetting("active_listening");
  const bindings = getSetting("bindings");
  const enabled = activeListening?.enabled ?? false;
  const hasActiveListeningShortcut = Boolean(
    bindings && (bindings as Record<string, unknown>)["active_listening"],
  );

  const handleEnableChange = async (value: boolean) => {
    await commands.changeActiveListeningEnabledSetting(value);
    await refreshSettings();
  };

  return (
    <div className="max-w-3xl w-full mx-auto space-y-6">
      <SettingsGroup title={t("settings.activeListening.general.title")}>
        <ToggleSwitch
          label={t("settings.activeListening.enable.title")}
          description={t("settings.activeListening.enable.description")}
          checked={enabled}
          onChange={handleEnableChange}
          grouped={true}
        />
        {enabled && hasActiveListeningShortcut && (
          <HandyShortcut shortcutId="active_listening" grouped={true} />
        )}
      </SettingsGroup>

      {enabled && (
        <>
          <SettingsGroup title={t("settings.activeListening.llm.title")}>
            <div className="p-3 bg-mid-gray/5 rounded-lg border border-mid-gray/20">
              <p className="text-sm text-mid-gray">
                {t(
                  "settings.activeListening.llm.centralizedNotice",
                  "AI provider is configured centrally in Settings > Advanced Features > AI Config. This section uses the shared configuration.",
                )}
              </p>
            </div>
          </SettingsGroup>

          <SettingsGroup title={t("settings.activeListening.segments.title")}>
            <SegmentSettingsComponent />
          </SettingsGroup>

          <AudioSourceSettings />

          <SettingsGroup title={t("settings.activeListening.prompts.title")}>
            <PromptsEditorComponent />
          </SettingsGroup>

          <SettingsGroup title={t("presentationMode.title")}>
            <SettingContainer
              title={t("presentationMode.description")}
              layout="horizontal"
              grouped={true}
            >
              <Button
                onClick={() => commands.openPresentationMode()}
                variant="primary"
                size="md"
              >
                <Presentation className="h-4 w-4 mr-2" />
                {t("presentationMode.open")}
              </Button>
            </SettingContainer>
          </SettingsGroup>

          <WakeWordSettings />

          <SettingsGroup
            title={t(
              "settings.activeListening.screenVision.title",
              "Screen Vision",
            )}
          >
            <ScreenVisionSettings />
          </SettingsGroup>

          <SpeakerAnalytics />

          <SessionViewer />
        </>
      )}

      {!enabled && (
        <DisabledNotice>
          {t("settings.activeListening.disabledNotice")}
        </DisabledNotice>
      )}
    </div>
  );
};
