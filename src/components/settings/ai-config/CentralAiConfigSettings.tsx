import React, { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { RefreshCcw } from "lucide-react";
import { commands, type LlmProvider } from "@/bindings";
import { Dropdown, SettingContainer } from "@/components/ui";
import { Input } from "@/components/ui/Input";
import { ResetButton } from "@/components/ui/ResetButton";
import { useSettings } from "@/hooks/useSettings";
import {
  type CloudProviderId,
  detectCloudProvider,
  prioritizeOpenRouterModels,
  resolveCloudFetchConfig,
} from "@/components/settings/active-listening/providerUtils";

export const CentralAiConfigSettings: React.FC = () => {
  const { t } = useTranslation();
  const { getSetting, refreshSettings } = useSettings();
  const [modelOptions, setModelOptions] = useState<string[]>([]);
  const [isFetchingModels, setIsFetchingModels] = useState(false);

  const activeListening = getSetting("active_listening");
  const provider: LlmProvider = activeListening?.llm_provider ?? "ollama";
  const ollamaBaseUrl = activeListening?.ollama_base_url ?? "http://localhost:11434";
  const ollamaModel = activeListening?.ollama_model ?? "";
  const llmModel = activeListening?.llm_model ?? "";
  const llmApiKey = activeListening?.llm_api_key ?? "";
  const llmBaseUrl = activeListening?.llm_base_url ?? "";
  const isLocalProvider = provider === "ollama" || provider === "local_open_ai";
  const mode = isLocalProvider ? "local" : "cloud";
  const cloudProvider = detectCloudProvider(provider, llmBaseUrl);

  const [ollamaBaseUrlDraft, setOllamaBaseUrlDraft] = useState(ollamaBaseUrl);
  const [llmApiKeyDraft, setLlmApiKeyDraft] = useState(llmApiKey);
  const [llmBaseUrlDraft, setLlmBaseUrlDraft] = useState(llmBaseUrl);
  const [llmModelDraft, setLlmModelDraft] = useState(llmModel);

  useEffect(() => setOllamaBaseUrlDraft(ollamaBaseUrl), [ollamaBaseUrl]);
  useEffect(() => setLlmApiKeyDraft(llmApiKey), [llmApiKey]);
  useEffect(() => setLlmBaseUrlDraft(llmBaseUrl), [llmBaseUrl]);
  useEffect(() => setLlmModelDraft(llmModel), [llmModel]);
  useEffect(() => {
    if (mode === "local" && provider === "ollama") {
      fetchOllamaModels();
    }
  }, [mode, provider, ollamaBaseUrl]);

  const fetchOllamaModels = async () => {
    setIsFetchingModels(true);
    try {
      const result = await commands.fetchOllamaModels();
      if (result.status === "ok") {
        setModelOptions(result.data.map((m) => m.name));
      }
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
        setModelOptions(
          cloudProvider === "open_router"
            ? prioritizeOpenRouterModels(result.data)
            : result.data,
        );
      }
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
    } finally {
      setIsFetchingModels(false);
    }
  };

  const refreshModelOptionsForCurrentProvider = async () => {
    if (mode === "local" && provider === "ollama") {
      await fetchOllamaModels();
      return;
    }
    if (mode === "local" && provider === "local_open_ai") {
      await fetchLocalOpenAiModels();
      return;
    }
    if (mode === "cloud") {
      await fetchCloudModels();
    }
  };

  return (
    <div className="space-y-2">
      <SettingContainer
        title={t("settings.sharedAi.mode.title", "Mode")}
        description={t(
          "settings.sharedAi.mode.description",
          "Choose where AI runs for all assistant features",
        )}
        descriptionMode="tooltip"
        layout="horizontal"
        grouped={true}
      >
        <div data-testid="shared-ai-mode">
          <Dropdown
            selectedValue={mode}
            options={[
              { value: "local", label: t("settings.activeListening.llm.mode.local") },
              { value: "cloud", label: t("settings.activeListening.llm.mode.cloud") },
            ]}
            onSelect={async (value) => {
              if (!value) return;
              await commands.changeActiveListeningLlmProvider(
                value === "local" ? "ollama" : "open_router",
              );
              await refreshSettings();
              setModelOptions([]);
            }}
            className="min-w-[220px]"
          />
        </div>
      </SettingContainer>

      {mode === "local" && (
        <>
          <SettingContainer
            title={t("settings.sharedAi.runtime.title", "Local Runtime")}
            descriptionMode="tooltip"
            layout="horizontal"
            grouped={true}
          >
            <div data-testid="shared-ai-runtime">
              <Dropdown
                selectedValue={provider === "local_open_ai" ? "local_open_ai" : "ollama"}
                options={[
                  {
                    value: "ollama",
                    label: t("settings.activeListening.llm.runtime.ollama"),
                  },
                  {
                    value: "local_open_ai",
                    label: t("settings.activeListening.llm.runtime.localOpenAi"),
                  },
                ]}
                onSelect={async (value) => {
                  if (!value) return;
                  await commands.changeActiveListeningLlmProvider(value as LlmProvider);
                  await refreshSettings();
                  setModelOptions([]);
                  if (value === "ollama") {
                    await fetchOllamaModels();
                  } else {
                    await fetchLocalOpenAiModels();
                  }
                }}
                className="min-w-[260px]"
              />
            </div>
          </SettingContainer>

          {provider === "ollama" ? (
            <>
              <SettingContainer
                title={t("settings.activeListening.ollama.baseUrl.title")}
                descriptionMode="tooltip"
                layout="horizontal"
                grouped={true}
              >
                <div data-testid="shared-ai-ollama-base-url">
                  <Input
                    value={ollamaBaseUrlDraft}
                    onChange={(e) => setOllamaBaseUrlDraft(e.target.value)}
                    onBlur={async () => {
                      await commands.changeOllamaBaseUrlSetting(ollamaBaseUrlDraft);
                      await refreshSettings();
                      await fetchOllamaModels();
                    }}
                    placeholder="http://localhost:11434"
                    className="min-w-[300px]"
                  />
                </div>
              </SettingContainer>
              <SettingContainer
                title={t("settings.activeListening.ollama.model.title")}
                descriptionMode="tooltip"
                layout="horizontal"
                grouped={true}
              >
                <div className="flex items-center gap-2" data-testid="shared-ai-ollama-model">
                  <Dropdown
                    selectedValue={ollamaModel || null}
                    options={modelOptions.map((m) => ({ value: m, label: m }))}
                    onSelect={async (newModel) => {
                      if (!newModel) return;
                      await commands.changeOllamaModelSetting(newModel);
                      await refreshSettings();
                    }}
                    placeholder={t("settings.activeListening.ollama.model.placeholder")}
                    disabled={isFetchingModels}
                    className="min-w-[250px]"
                  />
                  <ResetButton
                    onClick={fetchOllamaModels}
                    disabled={isFetchingModels}
                    ariaLabel={t("settings.activeListening.ollama.model.refresh")}
                    className="flex h-10 w-10 items-center justify-center"
                  >
                    <RefreshCcw className={`h-4 w-4 ${isFetchingModels ? "animate-spin" : ""}`} />
                  </ResetButton>
                </div>
              </SettingContainer>
            </>
          ) : (
            <>
              <SettingContainer
                title={t("settings.activeListening.llm.baseUrl.title")}
                descriptionMode="tooltip"
                layout="horizontal"
                grouped={true}
              >
                <Input
                  value={llmBaseUrlDraft}
                  onChange={(e) => setLlmBaseUrlDraft(e.target.value)}
                  onBlur={async () => {
                    await commands.changeActiveListeningLlmBaseUrl(llmBaseUrlDraft);
                    await refreshSettings();
                  }}
                  placeholder="http://127.0.0.1:8000/v1"
                  className="min-w-[300px]"
                />
              </SettingContainer>
              <SettingContainer
                title={t("settings.activeListening.llm.apiKey.title")}
                descriptionMode="tooltip"
                layout="horizontal"
                grouped={true}
              >
                <Input
                  type="password"
                  value={llmApiKeyDraft}
                  onChange={(e) => setLlmApiKeyDraft(e.target.value)}
                  onBlur={async () => {
                    await commands.changeActiveListeningLlmApiKey(llmApiKeyDraft);
                    await refreshSettings();
                    await fetchLocalOpenAiModels();
                  }}
                  placeholder="optional"
                  className="min-w-[300px]"
                />
              </SettingContainer>
            </>
          )}
        </>
      )}

      {mode === "cloud" && (
        <>
          <SettingContainer
            title={t("settings.activeListening.llm.provider.title")}
            descriptionMode="tooltip"
            layout="horizontal"
            grouped={true}
          >
            <div data-testid="shared-ai-cloud-provider">
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
                  { value: "groq", label: t("settings.activeListening.llm.provider.groq") },
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
                onSelect={async (value) => {
                  if (!value) return;
                  const selected = value as CloudProviderId;
                  await commands.changeActiveListeningLlmProvider(
                    selected === "custom" ? "custom" : (selected as LlmProvider),
                  );
                  await refreshSettings();
                  setModelOptions([]);
                  if (llmApiKeyDraft.trim()) {
                    await fetchCloudModels();
                  }
                }}
                className="min-w-[240px]"
              />
            </div>
          </SettingContainer>

          {cloudProvider === "custom" && (
            <SettingContainer
              title={t("settings.activeListening.llm.baseUrl.title")}
              descriptionMode="tooltip"
              layout="horizontal"
              grouped={true}
            >
              <div data-testid="shared-ai-base-url">
                <Input
                  value={llmBaseUrlDraft}
                  onChange={(e) => setLlmBaseUrlDraft(e.target.value)}
                  onBlur={async () => {
                    await commands.changeActiveListeningLlmBaseUrl(llmBaseUrlDraft);
                    await refreshSettings();
                  }}
                  placeholder="https://api.example.com/v1"
                  className="min-w-[300px]"
                />
              </div>
            </SettingContainer>
          )}

          <SettingContainer
            title={t("settings.activeListening.llm.apiKey.title")}
            descriptionMode="tooltip"
            layout="horizontal"
            grouped={true}
          >
            <div data-testid="shared-ai-api-key">
              <Input
                type="password"
                value={llmApiKeyDraft}
                onChange={(e) => setLlmApiKeyDraft(e.target.value)}
                onBlur={async () => {
                  await commands.changeActiveListeningLlmApiKey(llmApiKeyDraft);
                  await refreshSettings();
                  await fetchCloudModels();
                }}
                placeholder="sk-..."
                className="min-w-[300px]"
              />
            </div>
          </SettingContainer>
        </>
      )}

      <SettingContainer
        title={t("settings.sharedAi.model.title", "Default Model")}
        description={t(
          "settings.sharedAi.model.description",
          "Default model reused across assistant features unless overridden per feature",
        )}
        descriptionMode="tooltip"
        layout="horizontal"
        grouped={true}
      >
        <div className="flex items-center gap-2" data-testid="shared-ai-default-model">
          {modelOptions.length > 0 ? (
            <Dropdown
              selectedValue={
                mode === "local" && provider === "ollama" ? ollamaModel || null : llmModel || null
              }
              options={modelOptions.map((m) => ({ value: m, label: m }))}
              onSelect={async (newModel) => {
                if (!newModel) return;
                if (mode === "local" && provider === "ollama") {
                  await commands.changeOllamaModelSetting(newModel);
                } else {
                  await commands.changeActiveListeningLlmModel(newModel);
                }
                await refreshSettings();
              }}
              placeholder={t("settings.activeListening.llm.model.placeholder")}
              className="min-w-[250px]"
            />
          ) : (
            <Input
              value={mode === "local" && provider === "ollama" ? ollamaModel : llmModelDraft}
              onChange={(e) =>
                mode === "local" && provider === "ollama"
                  ? commands.changeOllamaModelSetting(e.target.value)
                  : setLlmModelDraft(e.target.value)
              }
              onBlur={async () => {
                if (mode !== "local" || provider !== "ollama") {
                  await commands.changeActiveListeningLlmModel(llmModelDraft);
                }
                await refreshSettings();
              }}
              placeholder={t("settings.activeListening.llm.model.inputPlaceholder")}
              className="min-w-[250px]"
            />
          )}
          <ResetButton
            onClick={refreshModelOptionsForCurrentProvider}
            disabled={isFetchingModels || (mode === "cloud" && !llmApiKeyDraft.trim())}
            ariaLabel={t("settings.activeListening.llm.model.refresh", "Refresh models")}
            className="flex h-10 w-10 items-center justify-center"
          >
            <RefreshCcw className={`h-4 w-4 ${isFetchingModels ? "animate-spin" : ""}`} />
          </ResetButton>
        </div>
      </SettingContainer>
    </div>
  );
};
