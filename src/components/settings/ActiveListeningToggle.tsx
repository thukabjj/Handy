import React from "react";
import { useTranslation } from "react-i18next";
import { ToggleSwitch } from "../ui/ToggleSwitch";
import { useSettings } from "../../hooks/useSettings";
import { commands } from "@/bindings";

interface ActiveListeningToggleProps {
  descriptionMode?: "inline" | "tooltip";
  grouped?: boolean;
}

export const ActiveListeningToggle: React.FC<ActiveListeningToggleProps> =
  React.memo(({ descriptionMode = "tooltip", grouped = false }) => {
    const { t } = useTranslation();
    const { getSetting, refreshSettings, isUpdating } = useSettings();

    const activeListening = getSetting("active_listening");
    const enabled = activeListening?.enabled ?? false;

    const handleChange = async (value: boolean) => {
      await commands.changeActiveListeningEnabledSetting(value);
      await refreshSettings();
    };

    return (
      <ToggleSwitch
        checked={enabled}
        onChange={handleChange}
        isUpdating={isUpdating("active_listening")}
        label={t("settings.advanced.activeListeningToggle.label")}
        description={t("settings.advanced.activeListeningToggle.description")}
        descriptionMode={descriptionMode}
        grouped={grouped}
      />
    );
  });
