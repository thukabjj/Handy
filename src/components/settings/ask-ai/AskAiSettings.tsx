import React, { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { RefreshCcw, Wifi, WifiOff } from "lucide-react";
import { commands } from "@/bindings";

import {
  Dropdown,
  SettingContainer,
  SettingsGroup,
  ToggleSwitch,
  Textarea,
} from "@/components/ui";
import { ResetButton } from "../../ui/ResetButton";
import { Input } from "../../ui/Input";
import { useSettings } from "../../../hooks/useSettings";
import { HandyShortcut } from "../HandyShortcut";
import { ConversationHistory } from "./ConversationHistory";

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
    ? t("settings.askAi.ollama.connectionStatus.checking")
    : isConnected
      ? t("settings.askAi.ollama.connectionStatus.connected")
      : t("settings.askAi.ollama.connectionStatus.disconnected");

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
        title={t("settings.askAi.ollama.refreshConnection")}
        aria-label={t("settings.askAi.ollama.refreshConnection")}
      >
        <RefreshCcw className={`h-4 w-4 ${isChecking ? "animate-spin" : ""}`} />
      </button>
    </div>
  );
};

const OllamaSettingsComponent: React.FC = () => {
  const { t } = useTranslation();
  const { getSetting, refreshSettings } = useSettings();
  const [modelOptions, setModelOptions] = useState<string[]>([]);
  const [isFetchingModels, setIsFetchingModels] = useState(false);

  const askAi = getSetting("ask_ai");
  const baseUrl = askAi?.ollama_base_url ?? "http://localhost:11434";
  const model = askAi?.ollama_model ?? "";

  const fetchModels = async () => {
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

  useEffect(() => {
    fetchModels();
  }, [baseUrl]);

  const handleBaseUrlChange = async (newUrl: string) => {
    if (!askAi) return;
    await commands.changeAskAiOllamaBaseUrlSetting(newUrl);
    await refreshSettings();
    fetchModels();
  };

  const handleModelChange = async (newModel: string | null) => {
    if (!askAi || !newModel) return;
    await commands.changeAskAiOllamaModelSetting(newModel);
    await refreshSettings();
  };

  return (
    <>
      <OllamaConnectionStatus />

      <SettingContainer
        title={t("settings.askAi.ollama.baseUrl.title")}
        description={t("settings.askAi.ollama.baseUrl.description")}
        descriptionMode="tooltip"
        layout="horizontal"
        grouped={true}
      >
        <Input
          value={baseUrl}
          onBlur={(e) => handleBaseUrlChange(e.target.value)}
          placeholder="http://localhost:11434"
          className="min-w-[300px]"
        />
      </SettingContainer>

      <SettingContainer
        title={t("settings.askAi.ollama.model.title")}
        description={t("settings.askAi.ollama.model.description")}
        descriptionMode="tooltip"
        layout="horizontal"
        grouped={true}
      >
        <div className="flex items-center gap-2">
          <Dropdown
            selectedValue={model || null}
            options={modelOptions.map((m) => ({ value: m, label: m }))}
            onSelect={handleModelChange}
            placeholder={
              modelOptions.length > 0
                ? t("settings.askAi.ollama.model.placeholder")
                : t("settings.askAi.ollama.model.placeholderNoModels")
            }
            disabled={isFetchingModels}
            className="min-w-[250px]"
          />
          <ResetButton
            onClick={fetchModels}
            disabled={isFetchingModels}
            ariaLabel={t("settings.askAi.ollama.model.refresh")}
            className="flex h-10 w-10 items-center justify-center"
          >
            <RefreshCcw
              className={`h-4 w-4 ${isFetchingModels ? "animate-spin" : ""}`}
            />
          </ResetButton>
        </div>
      </SettingContainer>
    </>
  );
};

const SystemPromptComponent: React.FC = () => {
  const { t } = useTranslation();
  const { getSetting, refreshSettings } = useSettings();

  const askAi = getSetting("ask_ai");
  const systemPrompt = askAi?.system_prompt ?? "";

  const handlePromptChange = async (newPrompt: string) => {
    await commands.changeAskAiSystemPromptSetting(newPrompt);
    await refreshSettings();
  };

  return (
    <SettingContainer
      title={t("settings.askAi.systemPrompt.title")}
      description={t("settings.askAi.systemPrompt.description")}
      descriptionMode="tooltip"
      layout="stacked"
      grouped={true}
    >
      <Textarea
        value={systemPrompt}
        onChange={(e) => handlePromptChange(e.target.value)}
        placeholder={t("settings.askAi.systemPrompt.placeholder")}
        rows={4}
      />
    </SettingContainer>
  );
};

export const AskAiSettings: React.FC = () => {
  const { t } = useTranslation();
  const { getSetting, refreshSettings } = useSettings();

  const askAi = getSetting("ask_ai");
  const enabled = askAi?.enabled ?? false;

  const handleEnableChange = async (value: boolean) => {
    await commands.changeAskAiEnabledSetting(value);
    await refreshSettings();
  };

  return (
    <div className="max-w-3xl w-full mx-auto space-y-6">
      <SettingsGroup title={t("settings.askAi.general.title")}>
        <ToggleSwitch
          label={t("settings.askAi.enable.title")}
          description={t("settings.askAi.enable.description")}
          checked={enabled}
          onChange={handleEnableChange}
          grouped={true}
        />
        {enabled && <HandyShortcut shortcutId="ask_ai" grouped={true} />}
      </SettingsGroup>

      {enabled && (
        <>
          <SettingsGroup title={t("settings.askAi.ollama.title")}>
            <OllamaSettingsComponent />
          </SettingsGroup>

          <SettingsGroup title={t("settings.askAi.systemPrompt.sectionTitle")}>
            <SystemPromptComponent />
          </SettingsGroup>

          <ConversationHistory />
        </>
      )}

      {!enabled && (
        <DisabledNotice>{t("settings.askAi.disabledNotice")}</DisabledNotice>
      )}
    </div>
  );
};
