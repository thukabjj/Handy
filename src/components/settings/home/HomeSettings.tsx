import React from "react";
import { useTranslation } from "react-i18next";
import { Mic, Keyboard, Globe } from "lucide-react";
import { SettingsGroup } from "../../ui/SettingsGroup";
import { SettingContainer } from "../../ui/SettingContainer";
import { ShortcutInput as HandyShortcut } from "../ShortcutInput";
import { LanguageSelector } from "../LanguageSelector";
import { PushToTalk } from "../PushToTalk";
import { useSettings } from "../../../hooks/useSettings";
import { useModelStore } from "../../../stores/modelStore";
import ModelSelector from "../../model-selector/ModelSelector";

/**
 * HomeSettings - The first thing users see.
 * Focused on getting transcribing as fast as possible.
 */
export const HomeSettings: React.FC = () => {
  const { t } = useTranslation();
  const { currentModel, getModelInfo } = useModelStore();
  const currentModelInfo = getModelInfo(currentModel);
  const showLanguageSelector = currentModelInfo?.engine_type === "Whisper";

  return (
    <div className="max-w-3xl w-full mx-auto space-y-6">
      {/* Model Status */}
      <SettingsGroup title={t("home.modelStatus")}>
        <div className="px-1">
          <ModelSelector />
        </div>
      </SettingsGroup>

      {/* Quick Start */}
      <SettingsGroup title={t("home.quickStart.title")}>
        <div className="space-y-3 px-4 py-2">
          <div className="flex items-start gap-3">
            <div className="flex items-center justify-center w-6 h-6 rounded-full bg-logo-primary/20 text-logo-primary text-xs font-bold shrink-0">
              1
            </div>
            <p className="text-sm text-text">{t("home.quickStart.step1")}</p>
          </div>
          <div className="flex items-start gap-3">
            <div className="flex items-center justify-center w-6 h-6 rounded-full bg-logo-primary/20 text-logo-primary text-xs font-bold shrink-0">
              2
            </div>
            <p className="text-sm text-text">{t("home.quickStart.step2")}</p>
          </div>
          <div className="flex items-start gap-3">
            <div className="flex items-center justify-center w-6 h-6 rounded-full bg-logo-primary/20 text-logo-primary text-xs font-bold shrink-0">
              3
            </div>
            <p className="text-sm text-text">{t("home.quickStart.step3")}</p>
          </div>
        </div>
      </SettingsGroup>

      {/* Shortcut */}
      <SettingsGroup title={t("settings.general.shortcut.title")}>
        <HandyShortcut shortcutId="transcribe" grouped={true} />
        <PushToTalk descriptionMode="tooltip" grouped={true} />
      </SettingsGroup>

      {/* Language (only for Whisper models) */}
      {showLanguageSelector && (
        <SettingsGroup title={t("settings.general.language.title")}>
          <LanguageSelector descriptionMode="tooltip" grouped={true} />
        </SettingsGroup>
      )}
    </div>
  );
};
