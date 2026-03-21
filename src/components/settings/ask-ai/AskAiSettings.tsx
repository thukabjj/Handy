import React from "react";
import { useTranslation } from "react-i18next";
import { commands } from "@/bindings";

import { SettingContainer, SettingsGroup, ToggleSwitch, Textarea } from "@/components/ui";
import { Input } from "../../ui/Input";
import { useSettings } from "../../../hooks/useSettings";
import { ShortcutInput as HandyShortcut } from "../ShortcutInput";
import { ConversationHistory } from "./ConversationHistory";

const DisabledNotice: React.FC<{ children: React.ReactNode }> = ({ children }) => (
  <div className="p-4 bg-mid-gray/5 rounded-lg border border-mid-gray/20">
    <p className="text-sm text-mid-gray">{children}</p>
  </div>
);

const AskAiModelOverrideSettings: React.FC = () => {
  const { t } = useTranslation();
  const { getSetting, refreshSettings } = useSettings();

  const askAi = getSetting("ask_ai");
  const activeListening = getSetting("active_listening");
  const provider = activeListening?.llm_provider ?? "ollama";
  const overrideModel =
    provider === "ollama" ? askAi?.ollama_model ?? "" : askAi?.llm_model ?? "";

  return (
    <>
      <div className="p-3 bg-mid-gray/5 rounded-lg border border-mid-gray/20">
        <p className="text-sm text-mid-gray">
          {t(
            "settings.askAi.llm.sharedConfigNotice",
            "AI provider is configured centrally in Settings > Advanced Features > AI Config.",
          )}
        </p>
      </div>

      <SettingContainer
        title={t("settings.askAi.llm.modelOverride.title", "Model Override")}
        description={t(
          "settings.askAi.llm.modelOverride.description",
          "Optional. Leave empty to use the centralized default model.",
        )}
        descriptionMode="tooltip"
        layout="horizontal"
        grouped={true}
      >
        <Input
          value={overrideModel}
          onChange={(e) =>
            provider === "ollama"
              ? commands.changeAskAiOllamaModelSetting(e.target.value)
              : commands.changeAskAiLlmModel(e.target.value)
          }
          onBlur={() => refreshSettings()}
          placeholder={t("settings.askAi.llm.model.inputPlaceholder")}
          className="min-w-[300px]"
        />
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
  const bindings = getSetting("bindings");
  const enabled = askAi?.enabled ?? false;
  const hasAskAiShortcut = Boolean(
    bindings && (bindings as Record<string, unknown>)["ask_ai"],
  );

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
        {enabled && hasAskAiShortcut && (
          <HandyShortcut shortcutId="ask_ai" grouped={true} />
        )}
      </SettingsGroup>

      {enabled && (
        <>
          <SettingsGroup title={t("settings.askAi.llm.title")}>
            <AskAiModelOverrideSettings />
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
