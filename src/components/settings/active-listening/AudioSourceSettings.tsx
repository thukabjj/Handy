import React, { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { AlertCircle, Monitor, Mic, Blend } from "lucide-react";
import { commands, AudioSourceType, LoopbackSupportLevel, LoopbackDeviceInfoDto } from "@/bindings";

import {
  Dropdown,
  SettingContainer,
  SettingsGroup,
  Slider,
} from "@/components/ui";
import { useSettings } from "../../../hooks/useSettings";

/**
 * Component to display loopback support status and any required notes
 */
const LoopbackSupportNotice: React.FC = () => {
  const { t } = useTranslation();
  const [supportLevel, setSupportLevel] = useState<LoopbackSupportLevel | null>(null);
  const [devices, setDevices] = useState<LoopbackDeviceInfoDto[]>([]);

  useEffect(() => {
    const checkSupport = async () => {
      try {
        const level = await commands.getLoopbackSupportLevel();
        setSupportLevel(level);

        if (level !== "not_supported") {
          const result = await commands.listLoopbackDevices();
          if (result.status === "ok") {
            setDevices(result.data);
          }
        }
      } catch (error) {
        console.error("Failed to check loopback support:", error);
      }
    };

    checkSupport();
  }, []);

  if (!supportLevel) return null;

  // Different messages based on support level
  if (supportLevel === "not_supported") {
    return (
      <div className="flex items-start gap-2 p-3 bg-amber-500/10 rounded-lg border border-amber-500/20">
        <AlertCircle className="h-4 w-4 text-amber-500 mt-0.5 flex-shrink-0" />
        <p className="text-sm text-amber-600 dark:text-amber-400">
          {t("settings.activeListening.audioSource.loopback.notAvailable")}
        </p>
      </div>
    );
  }

  if (supportLevel === "requires_virtual_device") {
    return (
      <div className="flex items-start gap-2 p-3 bg-blue-500/10 rounded-lg border border-blue-500/20">
        <AlertCircle className="h-4 w-4 text-blue-500 mt-0.5 flex-shrink-0" />
        <div className="text-sm text-blue-600 dark:text-blue-400">
          <p>{t("settings.activeListening.audioSource.loopback.macosNote")}</p>
          {devices.length === 0 && (
            <p className="mt-1 text-xs opacity-75">
              {t("settings.activeListening.audioSource.loopback.permissionRequired")}
            </p>
          )}
        </div>
      </div>
    );
  }

  return null;
};

/**
 * Audio source type selector with icons
 */
const AudioSourceTypeSelector: React.FC = () => {
  const { t } = useTranslation();
  const { getSetting, refreshSettings } = useSettings();
  const [isLoopbackSupported, setIsLoopbackSupported] = useState<boolean>(true);

  const activeListening = getSetting("active_listening");
  const currentType = activeListening?.audio_source_type ?? "microphone";

  useEffect(() => {
    const checkSupport = async () => {
      const supported = await commands.isLoopbackSupported();
      setIsLoopbackSupported(supported);
    };
    checkSupport();
  }, []);

  const handleTypeChange = async (value: string | null) => {
    if (!value) return;

    // Map dropdown value to AudioSourceType
    const sourceType = value as AudioSourceType;
    await commands.changeAudioSourceTypeSetting(sourceType);
    await refreshSettings();
  };

  const options = [
    {
      value: "microphone" as AudioSourceType,
      label: t("settings.activeListening.audioSource.type.microphone"),
      description: t("settings.activeListening.audioSource.type.microphoneDescription"),
      icon: Mic,
    },
    {
      value: "system_audio" as AudioSourceType,
      label: t("settings.activeListening.audioSource.type.systemAudio"),
      description: t("settings.activeListening.audioSource.type.systemAudioDescription"),
      icon: Monitor,
      disabled: !isLoopbackSupported,
    },
    {
      value: "mixed" as AudioSourceType,
      label: t("settings.activeListening.audioSource.type.mixed"),
      description: t("settings.activeListening.audioSource.type.mixedDescription"),
      icon: Blend,
      disabled: !isLoopbackSupported,
    },
  ];

  return (
    <SettingContainer
      title={t("settings.activeListening.audioSource.type.title")}
      description={t("settings.activeListening.audioSource.description")}
      descriptionMode="tooltip"
      layout="horizontal"
      grouped={true}
    >
      <Dropdown
        selectedValue={currentType}
        options={options.map((opt) => ({
          value: opt.value,
          label: opt.label,
          disabled: opt.disabled,
        }))}
        onSelect={handleTypeChange}
        className="min-w-[200px]"
      />
    </SettingContainer>
  );
};

/**
 * Mix ratio slider for balancing microphone and system audio
 */
const MixRatioSlider: React.FC = () => {
  const { t } = useTranslation();
  const { getSetting, refreshSettings } = useSettings();

  const activeListening = getSetting("active_listening");
  const mixRatio = activeListening?.audio_mix_settings?.mix_ratio ?? 0.5;
  const sourceType = activeListening?.audio_source_type ?? "microphone";

  // Only show when in mixed mode
  if (sourceType !== "mixed") {
    return null;
  }

  const handleRatioChange = async (value: number) => {
    await commands.changeAudioMixRatioSetting(value);
    await refreshSettings();
  };

  // Format the value to show as percentage with labels
  const formatValue = (value: number) => {
    const micPercent = Math.round((1 - value) * 100);
    const sysPercent = Math.round(value * 100);
    return `${micPercent}/${sysPercent}`;
  };

  return (
    <div className="space-y-2">
      <Slider
        value={mixRatio}
        onChange={handleRatioChange}
        min={0}
        max={1}
        step={0.05}
        label={t("settings.activeListening.audioSource.mixRatio.title")}
        description={t("settings.activeListening.audioSource.mixRatio.description")}
        descriptionMode="tooltip"
        grouped={true}
        showValue={true}
        formatValue={formatValue}
      />
      <div className="flex justify-between text-xs text-mid-gray px-1">
        <span>{t("settings.activeListening.audioSource.mixRatio.microphoneLabel")}</span>
        <span>{t("settings.activeListening.audioSource.mixRatio.systemLabel")}</span>
      </div>
    </div>
  );
};

/**
 * Main audio source settings component for Active Listening
 */
export const AudioSourceSettings: React.FC = () => {
  const { t } = useTranslation();

  return (
    <SettingsGroup title={t("settings.activeListening.audioSource.title")}>
      <LoopbackSupportNotice />
      <AudioSourceTypeSelector />
      <MixRatioSlider />
    </SettingsGroup>
  );
};
