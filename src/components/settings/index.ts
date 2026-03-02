// Settings section components
export { DebugSettings } from "./debug/DebugSettings";
export { HistorySettings } from "./history/HistorySettings";
export { AboutSettings } from "./about/AboutSettings";
export { PostProcessingSettings } from "./post-processing/PostProcessingSettings";
export { ActiveListeningSettings } from "./active-listening";
export { AskAiSettings } from "./ask-ai";
export { KnowledgeBaseSettings } from "./knowledge-base";
export { BatchProcessingPanel } from "./batch-processing";
export { VocabularyPanel } from "./vocabulary";

// New simplified navigation components
export { HomeSettings } from "./home";
export { LabsSettings } from "./labs";
export { UnifiedSettings } from "./unified";

// Shared components
export * from "./shared";

// Individual setting components
export { MicrophoneSelector } from "./MicrophoneSelector";
export { ClamshellMicrophoneSelector } from "./ClamshellMicrophoneSelector";
export { OutputDeviceSelector } from "./OutputDeviceSelector";
export { AlwaysOnMicrophone } from "./AlwaysOnMicrophone";
export { PushToTalk } from "./PushToTalk";
export { AudioFeedback } from "./AudioFeedback";
export { ShowOverlay } from "./ShowOverlay";
export { ShortcutInput as HandyShortcut } from "./ShortcutInput";
export { TranslateToEnglish } from "./TranslateToEnglish";
export { CustomWords } from "./CustomWords";
export { PostProcessingToggle } from "./PostProcessingToggle";
export { PostProcessingSettingsApi } from "./PostProcessingSettingsApi";
export { PostProcessingSettingsPrompts } from "./PostProcessingSettingsPrompts";
export { AppDataDirectory } from "./AppDataDirectory";
export { ModelUnloadTimeoutSetting } from "./ModelUnloadTimeout";
export { StartHidden } from "./StartHidden";
export { HistoryLimit } from "./HistoryLimit";
export { RecordingRetentionPeriodSelector } from "./RecordingRetentionPeriod";
export { AutostartToggle } from "./AutostartToggle";
export { UpdateChecksToggle } from "./UpdateChecksToggle";
export { LogDirectory } from "./LogDirectory";
export { PrivateOverlayToggle } from "./PrivateOverlayToggle";
export { AskAiToggle } from "./AskAiToggle";
export { KnowledgeBaseToggle } from "./KnowledgeBaseToggle";
