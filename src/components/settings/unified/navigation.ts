import type { SidebarSection } from "@/components/Sidebar";

export type UnifiedSection =
  | "audio"
  | "output"
  | "app"
  | "transcription"
  | "features";

export type UnifiedFeaturePanel =
  | "activeListening"
  | "askAi"
  | "knowledgeBase"
  | "batchProcessing"
  | "suggestions"
  | "aiConfig"
  | "diagnostics";

export interface SettingsNavigationTarget {
  section: SidebarSection;
  unifiedSection?: UnifiedSection;
  featurePanel?: UnifiedFeaturePanel;
}
