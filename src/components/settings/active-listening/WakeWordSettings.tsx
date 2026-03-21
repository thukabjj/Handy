import React, { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  commands,
  type WakeVoiceProfileStatus,
  type WakeWordAction,
} from "@/bindings";

import {
  Dropdown,
  SettingContainer,
  SettingsGroup,
  ToggleSwitch,
} from "@/components/ui";
import { Input } from "../../ui/Input";
import { useSettings } from "../../../hooks/useSettings";
import { HintRow } from "../shared/HintRow";
import { Button } from "../../ui/Button";

export const WakeWordSettings: React.FC = () => {
  const { t } = useTranslation();
  const { settings, refreshSettings } = useSettings();

  const wakeWord = settings?.wake_word;
  const enabled = wakeWord?.enabled ?? false;
  const phrase = wakeWord?.wake_phrase ?? "Hey Handy";
  const action = wakeWord?.trigger_action ?? "start_active_listening";
  const cooldown = wakeWord?.cooldown_seconds ?? 5;
  const kwsThreshold =
    wakeWord?.kws_threshold ?? wakeWord?.detection_threshold ?? 0.8;
  const spoofThreshold = wakeWord?.spoof_threshold ?? 0.55;
  const vadEnabled = wakeWord?.vad_enabled ?? true;
  const targetFar = wakeWord?.target_far_per_hour ?? 0.1;
  const voiceAuthEnabled = wakeWord?.voice_auth_enabled ?? false;
  const [profileStatus, setProfileStatus] =
    useState<WakeVoiceProfileStatus | null>(null);

  useEffect(() => {
    const loadStatus = async () => {
      try {
        const status = await commands.getWakeVoiceProfileStatus();
        setProfileStatus(status);
      } catch (error) {
        console.error("Failed to load wake voice profile status:", error);
      }
    };
    if (enabled) {
      loadStatus();
    }
  }, [enabled]);

  const handleEnabledChange = async (value: boolean) => {
    await commands.changeWakeWordEnabledSetting(value);
    await refreshSettings();
  };

  const handlePhraseChange = async (value: string) => {
    if (!value.trim()) return;
    await commands.changeWakePhraseSetting(value.trim());
    await refreshSettings();
  };

  const handleActionChange = async (value: string | null) => {
    if (!value) return;
    await commands.changeWakeWordActionSetting(value as WakeWordAction);
    await refreshSettings();
  };

  const handleCooldownChange = async (value: string | null) => {
    if (!value) return;
    await commands.changeWakeWordCooldownSetting(parseInt(value));
    await refreshSettings();
  };

  const handleVoiceAuthChange = async (value: boolean) => {
    await commands.changeWakeWordVoiceAuthEnabledSetting(value);
    await refreshSettings();
    const status = await commands.getWakeVoiceProfileStatus();
    setProfileStatus(status);
  };

  const handleKwsThresholdChange = async (value: string) => {
    const parsed = Number(value);
    if (Number.isNaN(parsed)) return;
    await commands.changeWakeWordKwsThresholdSetting(parsed);
    await refreshSettings();
  };

  const handleSpoofThresholdChange = async (value: string) => {
    const parsed = Number(value);
    if (Number.isNaN(parsed)) return;
    await commands.changeWakeWordSpoofThresholdSetting(parsed);
    await refreshSettings();
  };

  const handleVadChange = async (value: boolean) => {
    await commands.changeWakeWordVadEnabledSetting(value);
    await refreshSettings();
  };

  const handleFarChange = async (value: string) => {
    const parsed = Number(value);
    if (Number.isNaN(parsed)) return;
    await commands.changeWakeWordTargetFarSetting(parsed);
    await refreshSettings();
  };

  const handleAutoCalibrate = async () => {
    await commands.runWakeCalibrationSession();
    await refreshSettings();
  };

  return (
    <SettingsGroup title={t("settings.wakeWord.title")}>
      <ToggleSwitch
        label={t("settings.wakeWord.enable.title")}
        description={t("settings.wakeWord.enable.description")}
        checked={enabled}
        onChange={handleEnabledChange}
        grouped={true}
      />

      {enabled && (
        <>
          <SettingContainer
            title={t("settings.wakeWord.phrase.title")}
            description={t("settings.wakeWord.phrase.description")}
            descriptionMode="tooltip"
            layout="horizontal"
            grouped={true}
          >
            <Input
              value={phrase}
              onBlur={(e) => handlePhraseChange(e.target.value)}
              placeholder="Hey Handy"
              className="min-w-[200px]"
            />
          </SettingContainer>
          <HintRow
            text={t(
              "settings.wakeWord.hint",
              "Hint: speak the phrase naturally and keep background noise low for better wake accuracy.",
            )}
          />

          <SettingContainer
            title={t("settings.wakeWord.action.title")}
            description={t("settings.wakeWord.action.description")}
            descriptionMode="tooltip"
            layout="horizontal"
            grouped={true}
          >
            <Dropdown
              selectedValue={action}
              options={[
                {
                  value: "start_active_listening",
                  label: t("settings.wakeWord.action.startActiveListening"),
                },
                {
                  value: "start_recording",
                  label: t("settings.wakeWord.action.startRecording"),
                },
                {
                  value: "none",
                  label: t("settings.wakeWord.action.none"),
                },
              ]}
              onSelect={handleActionChange}
              className="min-w-[200px]"
            />
          </SettingContainer>

          <SettingContainer
            title={t(
              "settings.wakeWord.voiceAuth.title",
              "Recognize only my voice",
            )}
            description={t(
              "settings.wakeWord.voiceAuth.description",
              "Require an enrolled owner voice profile before wake actions run.",
            )}
            descriptionMode="tooltip"
            layout="horizontal"
            grouped={true}
          >
            <ToggleSwitch
              label=""
              description=""
              checked={voiceAuthEnabled}
              onChange={handleVoiceAuthChange}
              grouped={true}
            />
          </SettingContainer>

          {voiceAuthEnabled && (
            <div className="px-4 py-2 text-xs text-mid-gray border-t border-mid-gray/20">
              {profileStatus?.enrolled
                ? t(
                    "settings.wakeWord.voiceAuth.profileReady",
                    "Owner voice profile enrolled.",
                  )
                : t(
                    "settings.wakeWord.voiceAuth.profileMissing",
                    "No owner voice profile enrolled yet. Use start/capture/finish wake enrollment commands.",
                  )}
            </div>
          )}

          <SettingContainer
            title={t("settings.wakeWord.cooldown.title")}
            description={t("settings.wakeWord.cooldown.description")}
            descriptionMode="tooltip"
            layout="horizontal"
            grouped={true}
          >
            <Dropdown
              selectedValue={cooldown.toString()}
              options={[
                {
                  value: "2",
                  label: t("settings.wakeWord.cooldown.seconds", { count: 2 }),
                },
                {
                  value: "5",
                  label: t("settings.wakeWord.cooldown.seconds", { count: 5 }),
                },
                {
                  value: "10",
                  label: t("settings.wakeWord.cooldown.seconds", { count: 10 }),
                },
                {
                  value: "15",
                  label: t("settings.wakeWord.cooldown.seconds", { count: 15 }),
                },
                {
                  value: "30",
                  label: t("settings.wakeWord.cooldown.seconds", { count: 30 }),
                },
              ]}
              onSelect={handleCooldownChange}
            />
          </SettingContainer>

          <ToggleSwitch
            label={t("settings.wakeWord.vad.title", "VAD Gate")}
            description={t(
              "settings.wakeWord.vad.description",
              "Only evaluate wake phrase when speech activity is detected.",
            )}
            checked={vadEnabled}
            onChange={handleVadChange}
            grouped={true}
          />

          <SettingContainer
            title={t("settings.wakeWord.kwsThreshold.title", "KWS Threshold")}
            description={t(
              "settings.wakeWord.kwsThreshold.description",
              "Higher values reduce false triggers but may miss some wake attempts.",
            )}
            descriptionMode="tooltip"
            layout="horizontal"
            grouped={true}
          >
            <Input
              type="number"
              value={kwsThreshold.toString()}
              min="0"
              max="1"
              step="0.01"
              onBlur={(e) => handleKwsThresholdChange(e.target.value)}
              className="min-w-[120px]"
            />
          </SettingContainer>

          {voiceAuthEnabled && (
            <SettingContainer
              title={t(
                "settings.wakeWord.spoofThreshold.title",
                "Anti-Spoof Threshold",
              )}
              description={t(
                "settings.wakeWord.spoofThreshold.description",
                "Reject suspicious synthetic/replayed voice audio below this score.",
              )}
              descriptionMode="tooltip"
              layout="horizontal"
              grouped={true}
            >
              <Input
                type="number"
                value={spoofThreshold.toString()}
                min="0"
                max="1"
                step="0.01"
                onBlur={(e) => handleSpoofThresholdChange(e.target.value)}
                className="min-w-[120px]"
              />
            </SettingContainer>
          )}

          <SettingContainer
            title={t("settings.wakeWord.calibration.title", "Calibration")}
            description={t(
              "settings.wakeWord.calibration.description",
              "Tune thresholds from your target false accepts per hour.",
            )}
            descriptionMode="tooltip"
            layout="horizontal"
            grouped={true}
          >
            <div className="flex items-center gap-2">
              <Input
                type="number"
                value={targetFar.toString()}
                min="0.01"
                max="2"
                step="0.01"
                onBlur={(e) => handleFarChange(e.target.value)}
                className="min-w-[120px]"
              />
              <Button
                variant="secondary"
                size="sm"
                onClick={handleAutoCalibrate}
              >
                {t("settings.wakeWord.calibration.run", "Auto-calibrate")}
              </Button>
            </div>
          </SettingContainer>
        </>
      )}
    </SettingsGroup>
  );
};
