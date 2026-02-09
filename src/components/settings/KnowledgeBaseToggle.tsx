import React from "react";
import { useTranslation } from "react-i18next";
import { commands } from "@/bindings";
import { ToggleSwitch } from "@/components/ui";
import { useSettings } from "../../hooks/useSettings";

export const KnowledgeBaseToggle: React.FC = () => {
  const { t } = useTranslation();
  const { getSetting, refreshSettings } = useSettings();

  const knowledgeBase = getSetting("knowledge_base");
  const enabled = knowledgeBase?.enabled ?? false;

  const handleChange = async (value: boolean) => {
    const result = await commands.changeKnowledgeBaseEnabledSetting(value);
    if (result.status === "ok") {
      await refreshSettings();
    }
  };

  return (
    <ToggleSwitch
      label={t("settings.advanced.knowledgeBaseToggle.label")}
      description={t("settings.advanced.knowledgeBaseToggle.description")}
      checked={enabled}
      onChange={handleChange}
      grouped={true}
    />
  );
};
