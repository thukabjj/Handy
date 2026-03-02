import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { ChevronDown, Volume2, Settings2, BookA, Replace, Wrench } from "lucide-react";

// Audio settings
import { MicrophoneSelector } from "../MicrophoneSelector";
import { AudioFeedback } from "../AudioFeedback";
import { OutputDeviceSelector } from "../OutputDeviceSelector";
import { VolumeSlider } from "../VolumeSlider";
// SoundDetectionSettings temporarily removed - waiting for backend implementation
// import { SoundDetectionSettings } from "../shared/SoundDetectionSettings";

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

type Section = "audio" | "output" | "app" | "transcription" | null;

interface CollapsibleSectionProps {
  title: string;
  description: string;
  icon: React.ReactNode;
  expanded: boolean;
  onToggle: () => void;
  children: React.ReactNode;
}

const CollapsibleSection: React.FC<CollapsibleSectionProps> = ({
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
export const UnifiedSettings: React.FC = () => {
  const { t } = useTranslation();
  const { audioFeedbackEnabled, getSetting } = useSettings();
  const { currentModel, getModelInfo } = useModelStore();
  const currentModelInfo = getModelInfo(currentModel);
  const supportsTranslation = currentModelInfo?.supports_translation ?? false;

  const [expanded, setExpanded] = useState<Section>("audio");

  const toggleSection = (section: Section) => {
    setExpanded(expanded === section ? null : section);
  };

  return (
    <div className="max-w-3xl w-full mx-auto space-y-4">
      {/* Audio Settings */}
      <CollapsibleSection
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
        {/* SoundDetectionSettings temporarily removed - waiting for backend implementation */}
      </CollapsibleSection>

      {/* Output Settings */}
      <CollapsibleSection
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
    </div>
  );
};
