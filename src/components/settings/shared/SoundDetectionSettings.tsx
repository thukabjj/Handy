import React, { useCallback } from "react";
import { useTranslation } from "react-i18next";
import { commands, type SoundCategory } from "@/bindings";
import { SettingsGroup } from "@/components/ui/SettingsGroup";
import { ToggleSwitch } from "@/components/ui/ToggleSwitch";
import { Slider } from "@/components/ui/Slider";
import { useSettings } from "../../../hooks/useSettings";

const CATEGORIES = [
  "doorbell",
  "alarm",
  "phone_ring",
  "dog_bark",
  "baby_cry",
  "knocking",
  "siren",
  "applause",
] as const;

export const SoundDetectionSettings: React.FC = () => {
  const { t } = useTranslation();
  const { getSetting, refreshSettings } = useSettings();

  const soundDetection = getSetting("sound_detection");
  const enabled = soundDetection?.enabled ?? false;
  const threshold = soundDetection?.threshold ?? 0.5;
  const categories = soundDetection?.categories ?? [];
  const notificationEnabled = soundDetection?.notification_enabled ?? true;

  const handleEnabledChange = useCallback(
    async (value: boolean) => {
      const result = await commands.changeSoundDetectionEnabled(value);
      if (result.status === "ok") {
        await refreshSettings();
      }
    },
    [refreshSettings],
  );

  const handleThresholdChange = useCallback(
    async (value: number) => {
      const result = await commands.changeSoundDetectionThreshold(value);
      if (result.status === "ok") {
        await refreshSettings();
      }
    },
    [refreshSettings],
  );

  const handleCategoryToggle = useCallback(
    async (category: string, checked: boolean) => {
      const updated = (checked
        ? [...categories, category]
        : categories.filter((c: string) => c !== category)) as SoundCategory[];
      const result = await commands.changeSoundDetectionCategories(updated);
      if (result.status === "ok") {
        await refreshSettings();
      }
    },
    [categories, refreshSettings],
  );

  const handleNotificationChange = useCallback(
    async (value: boolean) => {
      const result = await commands.changeSoundDetectionNotification(value);
      if (result.status === "ok") {
        await refreshSettings();
      }
    },
    [refreshSettings],
  );

  return (
    <SettingsGroup title={t("soundDetection.title")}>
      <ToggleSwitch
        label={t("soundDetection.enabled")}
        description={t("soundDetection.enabledDescription")}
        checked={enabled}
        onChange={handleEnabledChange}
        grouped={true}
      />

      {enabled && (
        <>
          <div className="px-4 py-2">
            <div className="rounded-md bg-yellow-500/10 border border-yellow-500/20 px-3 py-2 text-xs text-yellow-200">
              {t("soundDetection.stubNotice")}
            </div>
          </div>

          <Slider
            label={t("soundDetection.threshold")}
            description={t("soundDetection.thresholdDescription")}
            value={threshold}
            min={0}
            max={1}
            step={0.05}
            onChange={handleThresholdChange}
          />

          <div className="px-4 py-3">
            <div className="text-sm font-medium text-neutral-200 mb-1">
              {t("soundDetection.categories")}
            </div>
            <p className="text-xs text-neutral-400 mb-3">
              {t("soundDetection.categoriesDescription")}
            </p>
            <div className="grid grid-cols-2 gap-2">
              {CATEGORIES.map((cat) => (
                <label
                  key={cat}
                  className="flex items-center gap-2 text-sm text-neutral-300 cursor-pointer"
                >
                  <input
                    type="checkbox"
                    checked={categories.includes(cat)}
                    onChange={(e) => handleCategoryToggle(cat, e.target.checked)}
                    className="rounded border-neutral-600 bg-neutral-800 text-blue-500 focus:ring-blue-500/20"
                  />
                  {t(`soundDetection.category.${cat}`)}
                </label>
              ))}
            </div>
          </div>

          <ToggleSwitch
            label={t("soundDetection.notification")}
            description={t("soundDetection.notificationDescription")}
            checked={notificationEnabled}
            onChange={handleNotificationChange}
            grouped={true}
          />
        </>
      )}
    </SettingsGroup>
  );
};
