import React from "react";
import { useTranslation } from "react-i18next";
import { ToggleSwitch } from "../ui/ToggleSwitch";
import { useSettings } from "../../hooks/useSettings";

interface PrivateOverlayToggleProps {
  descriptionMode?: "inline" | "tooltip";
  grouped?: boolean;
}

export const PrivateOverlayToggle: React.FC<PrivateOverlayToggleProps> = ({
  descriptionMode = "tooltip",
  grouped = false,
}) => {
  const { t } = useTranslation();
  const { getSetting, updateSetting, isUpdating } = useSettings();
  const privateOverlayEnabled = getSetting("private_overlay") ?? true;

  return (
    <ToggleSwitch
      checked={privateOverlayEnabled}
      onChange={(enabled) => updateSetting("private_overlay", enabled)}
      isUpdating={isUpdating("private_overlay")}
      label={t("settings.advanced.privateOverlay.label")}
      description={t("settings.advanced.privateOverlay.description")}
      descriptionMode={descriptionMode}
      grouped={grouped}
    />
  );
};
