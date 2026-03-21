import React, { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  ChevronDown,
  Volume2,
  Settings2,
  BookA,
  Wrench,
  Database,
  ListChecks,
  Sparkles,
  Bot,
  Activity,
} from "lucide-react";
import { commands } from "@/bindings";

// Audio settings
import { MicrophoneSelector } from "../MicrophoneSelector";
import { AudioFeedback } from "../AudioFeedback";
import { OutputDeviceSelector } from "../OutputDeviceSelector";
import { VolumeSlider } from "../VolumeSlider";
import { SoundDetectionSettings } from "../shared/SoundDetectionSettings";

// Output settings
import { PasteMethodSetting } from "../PasteMethod";
import { TypingToolSetting } from "../TypingTool";
import { ClipboardHandlingSetting } from "../ClipboardHandling";
import { AutoSubmit } from "../AutoSubmit";
import { AppendTrailingSpace } from "../AppendTrailingSpace";

// App settings
import { StartHidden } from "../StartHidden";
import { AutostartToggle } from "../AutostartToggle";
import { ShowTrayIcon } from "../ShowTrayIcon";
import { ShowOverlay } from "../ShowOverlay";
import { ModelUnloadTimeoutSetting } from "../ModelUnloadTimeout";
import { ExperimentalToggle } from "../ExperimentalToggle";
import { HistoryLimit } from "../HistoryLimit";
import { RecordingRetentionPeriodSelector } from "../RecordingRetentionPeriod";

// Transcription settings
import { CustomWords } from "../CustomWords";
import { TranslateToEnglish } from "../TranslateToEnglish";

import { SettingsGroup } from "../../ui/SettingsGroup";
import { useSettings } from "../../../hooks/useSettings";
import { useModelStore } from "../../../stores/modelStore";
import { Button } from "../../ui/Button";
import { KnowledgeBaseSettings } from "../knowledge-base";
import { BatchProcessingPanel } from "../batch-processing";
import { SuggestionsSettings } from "../suggestions";
import { CentralAiConfigSettings } from "../ai-config/CentralAiConfigSettings";
import { ActiveListeningSettings } from "../active-listening";
import { AskAiSettings } from "../ask-ai";
import { DiagnosticsSettings } from "../diagnostics";
import type {
  SettingsNavigationTarget,
  UnifiedFeaturePanel,
  UnifiedSection,
} from "./navigation";

type Section = UnifiedSection | null;
type FeaturePanel = UnifiedFeaturePanel | null;

interface CollapsibleSectionProps {
  testId: string;
  title: string;
  description: string;
  icon: React.ReactNode;
  expanded: boolean;
  onToggle: () => void;
  children: React.ReactNode;
}

const CollapsibleSection: React.FC<CollapsibleSectionProps> = ({
  testId,
  title,
  description,
  icon,
  expanded,
  onToggle,
  children,
}) => {
  return (
    <div className="rounded-lg border border-mid-gray/20 overflow-hidden">
      <button
        onClick={onToggle}
        data-testid={testId}
        aria-expanded={expanded}
        className="w-full p-4 flex items-center gap-3 hover:bg-mid-gray/5 transition-colors text-left"
      >
        <div className="text-mid-gray">{icon}</div>
        <div className="flex-1 min-w-0">
          <h3 className="font-medium text-text">{title}</h3>
          <p className="text-sm text-mid-gray">{description}</p>
        </div>
        <ChevronDown
          className={`h-5 w-5 text-mid-gray transition-transform ${expanded ? "rotate-180" : ""}`}
        />
      </button>
      {expanded && (
        <div className="px-4 pb-4 border-t border-mid-gray/20 bg-mid-gray/5">
          <div className="pt-4 space-y-2">{children}</div>
        </div>
      )}
    </div>
  );
};

/**
 * UnifiedSettings - Consolidated app settings with collapsible sections.
 */
interface UnifiedSettingsProps {
  navigationTarget?: SettingsNavigationTarget | null;
  onNavigationTargetApplied?: () => void;
}

export const UnifiedSettings: React.FC<UnifiedSettingsProps> = ({
  navigationTarget,
  onNavigationTargetApplied,
}) => {
  const { t } = useTranslation();
  const { audioFeedbackEnabled, settings, refreshSettings } = useSettings();
  const { currentModel, getModelInfo } = useModelStore();
  const currentModelInfo = getModelInfo(currentModel);
  const supportsTranslation = currentModelInfo?.supports_translation ?? false;

  const [expanded, setExpanded] = useState<Section>("audio");
  const [activeFeaturePanel, setActiveFeaturePanel] =
    useState<FeaturePanel>(null);

  const toggleSection = (section: Section) => {
    setExpanded(expanded === section ? null : section);
  };

  useEffect(() => {
    if (!navigationTarget || navigationTarget.section !== "settings") {
      return;
    }

    if (navigationTarget.unifiedSection) {
      setExpanded(navigationTarget.unifiedSection);
    }

    if (navigationTarget.featurePanel) {
      setExpanded("features");
      setActiveFeaturePanel(navigationTarget.featurePanel);
    } else if (
      navigationTarget.unifiedSection &&
      navigationTarget.unifiedSection !== "features"
    ) {
      setActiveFeaturePanel(null);
    }

    onNavigationTargetApplied?.();
  }, [navigationTarget, onNavigationTargetApplied]);

  const handleKnowledgeBaseToggle = async (value: boolean) => {
    await commands.changeKnowledgeBaseEnabledSetting(value);
    await refreshSettings();
  };

  const handleSuggestionsToggle = async (value: boolean) => {
    await commands.changeSuggestionsEnabledSetting(value);
    await refreshSettings();
  };

  const handleActiveListeningToggle = async (value: boolean) => {
    await commands.changeActiveListeningEnabledSetting(value);
    await refreshSettings();
  };

  const handleAskAiToggle = async (value: boolean) => {
    await commands.changeAskAiEnabledSetting(value);
    await refreshSettings();
  };

  return (
    <div className="max-w-3xl w-full mx-auto space-y-4">
      {/* Audio Settings */}
      <CollapsibleSection
        testId="unified-section-audio-toggle"
        title={t("unifiedSettings.audio.title")}
        description={t("unifiedSettings.audio.description")}
        icon={<Volume2 className="h-5 w-5" />}
        expanded={expanded === "audio"}
        onToggle={() => toggleSection("audio")}
      >
        <MicrophoneSelector descriptionMode="tooltip" grouped={true} />
        <AudioFeedback descriptionMode="tooltip" grouped={true} />
        <OutputDeviceSelector
          descriptionMode="tooltip"
          grouped={true}
          disabled={!audioFeedbackEnabled}
        />
        <VolumeSlider disabled={!audioFeedbackEnabled} />
        <SoundDetectionSettings />
      </CollapsibleSection>

      {/* Output Settings */}
      <CollapsibleSection
        testId="unified-section-output-toggle"
        title={t("unifiedSettings.output.title")}
        description={t("unifiedSettings.output.description")}
        icon={<Settings2 className="h-5 w-5" />}
        expanded={expanded === "output"}
        onToggle={() => toggleSection("output")}
      >
        <PasteMethodSetting descriptionMode="tooltip" grouped={true} />
        <TypingToolSetting descriptionMode="tooltip" grouped={true} />
        <ClipboardHandlingSetting descriptionMode="tooltip" grouped={true} />
        <AutoSubmit descriptionMode="tooltip" grouped={true} />
        <AppendTrailingSpace descriptionMode="tooltip" grouped={true} />
      </CollapsibleSection>

      {/* Transcription Settings */}
      <CollapsibleSection
        testId="unified-section-transcription-toggle"
        title={t("unifiedSettings.vocabulary.title")}
        description={t("unifiedSettings.vocabulary.description")}
        icon={<BookA className="h-5 w-5" />}
        expanded={expanded === "transcription"}
        onToggle={() => toggleSection("transcription")}
      >
        <CustomWords descriptionMode="tooltip" grouped />
        {supportsTranslation && (
          <TranslateToEnglish descriptionMode="tooltip" grouped={true} />
        )}
      </CollapsibleSection>

      {/* App Settings (Advanced) */}
      <CollapsibleSection
        testId="unified-section-app-toggle"
        title={t("unifiedSettings.advanced.title")}
        description={t("unifiedSettings.advanced.description")}
        icon={<Wrench className="h-5 w-5" />}
        expanded={expanded === "app"}
        onToggle={() => toggleSection("app")}
      >
        <StartHidden descriptionMode="tooltip" grouped={true} />
        <AutostartToggle descriptionMode="tooltip" grouped={true} />
        <ShowTrayIcon descriptionMode="tooltip" grouped={true} />
        <ShowOverlay descriptionMode="tooltip" grouped={true} />
        <ModelUnloadTimeoutSetting descriptionMode="tooltip" grouped={true} />
        <HistoryLimit descriptionMode="tooltip" grouped={true} />
        <RecordingRetentionPeriodSelector
          descriptionMode="tooltip"
          grouped={true}
        />
        <ExperimentalToggle descriptionMode="tooltip" grouped={true} />
      </CollapsibleSection>

      <CollapsibleSection
        testId="unified-section-features-toggle"
        title={t("unifiedSettings.features.title")}
        description={t("unifiedSettings.features.description")}
        icon={<Database className="h-5 w-5" />}
        expanded={expanded === "features"}
        onToggle={() => toggleSection("features")}
      >
        <div className="space-y-3">
          <div
            className="rounded-lg border border-mid-gray/20 bg-mid-gray/5 p-3"
            data-testid="feature-card-ai-config"
          >
            <div className="flex items-center justify-between gap-3">
              <div className="flex items-center gap-2">
                <Bot className="h-4 w-4 text-mid-gray" />
                <div>
                  <p className="text-sm font-medium">
                    {t("unifiedSettings.features.aiConfig.title", "AI Config")}
                  </p>
                  <p className="text-xs text-mid-gray">
                    {t(
                      "unifiedSettings.features.aiConfig.description",
                      "Configure provider and default model once for all assistant features",
                    )}
                  </p>
                </div>
              </div>
              <Button
                variant="secondary"
                size="sm"
                data-testid="feature-configure-ai-config"
                onClick={() =>
                  setActiveFeaturePanel((prev) =>
                    prev === "aiConfig" ? null : "aiConfig",
                  )
                }
              >
                {t("unifiedSettings.features.configure")}
              </Button>
            </div>
          </div>
          {activeFeaturePanel === "aiConfig" && (
            <div className="mt-3 rounded-lg border border-mid-gray/20 bg-background/40 p-3">
              <CentralAiConfigSettings />
            </div>
          )}

          <div
            className="rounded-lg border border-mid-gray/20 bg-mid-gray/5 p-3"
            data-testid="feature-card-knowledge-base"
          >
            <div className="flex items-center justify-between gap-3">
              <div className="flex items-center gap-2">
                <Database className="h-4 w-4 text-mid-gray" />
                <div>
                  <p className="text-sm font-medium">
                    {t("unifiedSettings.features.knowledgeBase.title")}
                  </p>
                  <p className="text-xs text-mid-gray">
                    {t("unifiedSettings.features.knowledgeBase.description")}
                  </p>
                </div>
              </div>
              <div className="flex items-center gap-2">
                <label className="flex items-center gap-2 text-xs text-mid-gray">
                  <input
                    type="checkbox"
                    checked={settings?.knowledge_base?.enabled ?? false}
                    onChange={(e) => handleKnowledgeBaseToggle(e.target.checked)}
                    className="rounded border-mid-gray/40"
                  />
                  {t("common.enabled", "Enabled")}
                </label>
                <Button
                  variant="secondary"
                  size="sm"
                  data-testid="feature-configure-knowledge-base"
                  onClick={() =>
                    setActiveFeaturePanel((prev) =>
                      prev === "knowledgeBase" ? null : "knowledgeBase",
                    )
                  }
                >
                  {t("unifiedSettings.features.configure")}
                </Button>
              </div>
            </div>
          </div>
          {activeFeaturePanel === "knowledgeBase" && (
            <div className="mt-3 rounded-lg border border-mid-gray/20 bg-background/40 p-3">
              <KnowledgeBaseSettings />
            </div>
          )}

          <div
            className="rounded-lg border border-mid-gray/20 bg-mid-gray/5 p-3"
            data-testid="feature-card-active-listening"
          >
            <div className="flex items-center justify-between gap-3">
              <div className="flex items-center gap-2">
                <Database className="h-4 w-4 text-mid-gray" />
                <div>
                  <p className="text-sm font-medium">
                    {t("settings.activeListening.general.title", "Active Listening")}
                  </p>
                  <p className="text-xs text-mid-gray">
                    {t(
                      "settings.activeListening.enable.description",
                      "Continuously transcribe audio and generate AI insights",
                    )}
                  </p>
                </div>
              </div>
              <div className="flex items-center gap-2">
                <label className="flex items-center gap-2 text-xs text-mid-gray">
                  <input
                    type="checkbox"
                    checked={settings?.active_listening?.enabled ?? false}
                    onChange={(e) => handleActiveListeningToggle(e.target.checked)}
                    className="rounded border-mid-gray/40"
                  />
                  {t("common.enabled", "Enabled")}
                </label>
                <Button
                  variant="secondary"
                  size="sm"
                  data-testid="feature-configure-active-listening"
                  onClick={() =>
                    setActiveFeaturePanel((prev) =>
                      prev === "activeListening" ? null : "activeListening",
                    )
                  }
                >
                  {t("unifiedSettings.features.configure")}
                </Button>
              </div>
            </div>
          </div>
          {activeFeaturePanel === "activeListening" && (
            <div className="mt-3 rounded-lg border border-mid-gray/20 bg-background/40 p-3">
              <ActiveListeningSettings />
            </div>
          )}

          <div
            className="rounded-lg border border-mid-gray/20 bg-mid-gray/5 p-3"
            data-testid="feature-card-ask-ai"
          >
            <div className="flex items-center justify-between gap-3">
              <div className="flex items-center gap-2">
                <Sparkles className="h-4 w-4 text-mid-gray" />
                <div>
                  <p className="text-sm font-medium">
                    {t("settings.askAi.general.title", "Ask AI")}
                  </p>
                  <p className="text-xs text-mid-gray">
                    {t(
                      "settings.askAi.enable.description",
                      "Multi-turn voice conversations with your LLM",
                    )}
                  </p>
                </div>
              </div>
              <div className="flex items-center gap-2">
                <label className="flex items-center gap-2 text-xs text-mid-gray">
                  <input
                    type="checkbox"
                    checked={settings?.ask_ai?.enabled ?? false}
                    onChange={(e) => handleAskAiToggle(e.target.checked)}
                    className="rounded border-mid-gray/40"
                  />
                  {t("common.enabled", "Enabled")}
                </label>
                <Button
                  variant="secondary"
                  size="sm"
                  data-testid="feature-configure-ask-ai"
                  onClick={() =>
                    setActiveFeaturePanel((prev) =>
                      prev === "askAi" ? null : "askAi",
                    )
                  }
                >
                  {t("unifiedSettings.features.configure")}
                </Button>
              </div>
            </div>
          </div>
          {activeFeaturePanel === "askAi" && (
            <div className="mt-3 rounded-lg border border-mid-gray/20 bg-background/40 p-3">
              <AskAiSettings />
            </div>
          )}

          <div
            className="rounded-lg border border-mid-gray/20 bg-mid-gray/5 p-3"
            data-testid="feature-card-batch-processing"
          >
            <div className="flex items-center justify-between gap-3">
              <div className="flex items-center gap-2">
                <ListChecks className="h-4 w-4 text-mid-gray" />
                <div>
                  <p className="text-sm font-medium">
                    {t("unifiedSettings.features.batchProcessing.title")}
                  </p>
                  <p className="text-xs text-mid-gray">
                    {t("unifiedSettings.features.batchProcessing.description")}
                  </p>
                </div>
              </div>
              <Button
                variant="secondary"
                size="sm"
                data-testid="feature-configure-batch-processing"
                onClick={() =>
                  setActiveFeaturePanel((prev) =>
                    prev === "batchProcessing" ? null : "batchProcessing",
                  )
                }
              >
                {t("unifiedSettings.features.configure")}
              </Button>
            </div>
          </div>
          {activeFeaturePanel === "batchProcessing" && (
            <div className="mt-3 rounded-lg border border-mid-gray/20 bg-background/40 p-3">
              <BatchProcessingPanel />
            </div>
          )}

          <div
            className="rounded-lg border border-mid-gray/20 bg-mid-gray/5 p-3"
            data-testid="feature-card-suggestions"
          >
            <div className="flex items-center justify-between gap-3">
              <div className="flex items-center gap-2">
                <Sparkles className="h-4 w-4 text-mid-gray" />
                <div>
                  <p className="text-sm font-medium">
                    {t("unifiedSettings.features.suggestions.title")}
                  </p>
                  <p className="text-xs text-mid-gray">
                    {t("unifiedSettings.features.suggestions.description")}
                  </p>
                </div>
              </div>
              <div className="flex items-center gap-2">
                <label className="flex items-center gap-2 text-xs text-mid-gray">
                  <input
                    type="checkbox"
                    checked={settings?.suggestions?.enabled ?? false}
                    onChange={(e) => handleSuggestionsToggle(e.target.checked)}
                    className="rounded border-mid-gray/40"
                  />
                  {t("common.enabled", "Enabled")}
                </label>
                <Button
                  variant="secondary"
                  size="sm"
                  data-testid="feature-configure-suggestions"
                  onClick={() =>
                    setActiveFeaturePanel((prev) =>
                      prev === "suggestions" ? null : "suggestions",
                    )
                  }
                >
                  {t("unifiedSettings.features.configure")}
                </Button>
              </div>
            </div>
          </div>
          {activeFeaturePanel === "suggestions" && (
            <div className="mt-3 rounded-lg border border-mid-gray/20 bg-background/40 p-3">
              <SuggestionsSettings />
            </div>
          )}

          <div
            className="rounded-lg border border-mid-gray/20 bg-mid-gray/5 p-3"
            data-testid="feature-card-diagnostics"
          >
            <div className="flex items-center justify-between gap-3">
              <div className="flex items-center gap-2">
                <Activity className="h-4 w-4 text-mid-gray" />
                <div>
                  <p className="text-sm font-medium">
                    {t("diagnostics.title", "Observability")}
                  </p>
                  <p className="text-xs text-mid-gray">
                    {t(
                      "diagnostics.descriptionShort",
                      "Live diagnostics for wake, audio, Ask AI, screen vision, and provider traffic.",
                    )}
                  </p>
                </div>
              </div>
              <Button
                variant="secondary"
                size="sm"
                data-testid="feature-configure-diagnostics"
                onClick={() =>
                  setActiveFeaturePanel((prev) =>
                    prev === "diagnostics" ? null : "diagnostics",
                  )
                }
              >
                {t("unifiedSettings.features.configure")}
              </Button>
            </div>
          </div>
          {activeFeaturePanel === "diagnostics" && (
            <div className="mt-3 rounded-lg border border-mid-gray/20 bg-background/40 p-3">
              <DiagnosticsSettings />
            </div>
          )}
        </div>
      </CollapsibleSection>
    </div>
  );
};
