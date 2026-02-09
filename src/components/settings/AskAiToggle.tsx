import React from "react";
import { useTranslation } from "react-i18next";
import { ToggleSwitch } from "../ui/ToggleSwitch";
import { useSettings } from "../../hooks/useSettings";
import { commands } from "@/bindings";

interface AskAiToggleProps {
  descriptionMode?: "inline" | "tooltip";
  grouped?: boolean;
}

export const AskAiToggle: React.FC<AskAiToggleProps> = React.memo(
  ({ descriptionMode = "tooltip", grouped = false }) => {
    const { t } = useTranslation();
    const { getSetting, refreshSettings, isUpdating } = useSettings();

    const askAi = getSetting("ask_ai");
    const enabled = askAi?.enabled ?? false;

    const handleChange = async (value: boolean) => {
      await commands.changeAskAiEnabledSetting(value);
      await refreshSettings();
    };

    return (
      <ToggleSwitch
        checked={enabled}
        onChange={handleChange}
        isUpdating={isUpdating("ask_ai")}
        label={t("settings.advanced.askAiToggle.label")}
        description={t("settings.advanced.askAiToggle.description")}
        descriptionMode={descriptionMode}
        grouped={grouped}
      />
    );
  }
);
