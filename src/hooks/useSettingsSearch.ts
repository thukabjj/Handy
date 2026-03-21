import { useMemo, useEffect, useState, useCallback } from "react";
import Fuse from "fuse.js";
import { useTranslation } from "react-i18next";
import { SECTIONS_CONFIG, type SidebarSection } from "@/components/Sidebar";
import { useSettings } from "@/hooks/useSettings";
import type { SettingsNavigationTarget } from "@/components/settings/unified/navigation";

export interface SearchableItem {
  id: string;
  sectionLabel: string;
  label: string;
  description: string;
  keywords: string[];
  target: SettingsNavigationTarget;
}

interface UseSettingsSearchReturn {
  results: SearchableItem[];
  query: string;
  setQuery: (query: string) => void;
  isOpen: boolean;
  setIsOpen: (open: boolean) => void;
}

/**
 * Map of section keys to their searchable settings entries.
 * Each entry has a translation key prefix and optional extra keywords.
 *
 * Sections:
 * - home: Quick start, shortcut, language
 * - settings: Audio, output, app settings
 * - settings: Core + advanced features
 * - history: Transcription history
 * - about: Version, source code
 * - debug: Log level, updates (only visible in debug mode)
 */
const SEARCHABLE_ENTRIES: Record<
  string,
  { target: SettingsNavigationTarget; keyPrefix: string; keywords?: string[] }[]
> = {
  home: [
    {
      target: { section: "home" },
      keyPrefix: "settings.general.shortcut",
      keywords: ["hotkey", "keybind", "keyboard"],
    },
    {
      target: { section: "home" },
      keyPrefix: "settings.general.language",
      keywords: ["locale", "speech recognition"],
    },
    {
      target: { section: "home" },
      keyPrefix: "settings.general.pushToTalk",
      keywords: ["hold", "record"],
    },
  ],
  settings: [
    {
      target: { section: "settings", unifiedSection: "audio" },
      keyPrefix: "settings.sound.microphone",
      keywords: ["mic", "input device", "audio input"],
    },
    {
      target: { section: "settings", unifiedSection: "audio" },
      keyPrefix: "settings.sound.audioFeedback",
      keywords: ["sound", "beep", "notification"],
    },
    {
      target: { section: "settings", unifiedSection: "audio" },
      keyPrefix: "settings.sound.outputDevice",
      keywords: ["speaker", "audio output"],
    },
    {
      target: { section: "settings", unifiedSection: "audio" },
      keyPrefix: "settings.sound.volume",
      keywords: ["loudness", "level"],
    },
    {
      target: { section: "settings", unifiedSection: "app" },
      keyPrefix: "settings.advanced.startHidden",
      keywords: ["tray", "minimize", "launch"],
    },
    {
      target: { section: "settings", unifiedSection: "app" },
      keyPrefix: "settings.advanced.autostart",
      keywords: ["login", "boot", "startup"],
    },
    {
      target: { section: "settings", unifiedSection: "app" },
      keyPrefix: "settings.advanced.overlay",
      keywords: ["position", "visual", "feedback"],
    },
    {
      target: { section: "settings", unifiedSection: "output" },
      keyPrefix: "settings.advanced.pasteMethod",
      keywords: ["clipboard", "typing", "insert"],
    },
    {
      target: { section: "settings", unifiedSection: "output" },
      keyPrefix: "settings.advanced.clipboardHandling",
      keywords: ["copy", "paste"],
    },
    {
      target: { section: "settings", unifiedSection: "transcription" },
      keyPrefix: "settings.advanced.translateToEnglish",
      keywords: ["translation", "language"],
    },
    {
      target: { section: "settings", unifiedSection: "app" },
      keyPrefix: "settings.advanced.modelUnload",
      keywords: ["memory", "gpu", "unload"],
    },
    {
      target: { section: "settings", unifiedSection: "transcription" },
      keyPrefix: "settings.advanced.customWords",
      keywords: ["dictionary", "correction", "spelling", "vocabulary"],
    },
  ],
  settingsAdvanced: [
    {
      target: {
        section: "settings",
        unifiedSection: "features",
        featurePanel: "knowledgeBase",
      },
      keyPrefix: "unifiedSettings.features.knowledgeBase",
      keywords: ["rag", "documents", "embeddings", "search"],
    },
    {
      target: {
        section: "settings",
        unifiedSection: "features",
        featurePanel: "batchProcessing",
      },
      keyPrefix: "unifiedSettings.features.batchProcessing",
      keywords: ["batch", "queue", "import", "transcription files"],
    },
    {
      target: {
        section: "settings",
        unifiedSection: "features",
        featurePanel: "suggestions",
      },
      keyPrefix: "unifiedSettings.features.suggestions",
      keywords: [
        "coaching",
        "quick responses",
        "rag suggestions",
        "talking points",
      ],
    },
    {
      target: {
        section: "settings",
        unifiedSection: "features",
        featurePanel: "aiConfig",
      },
      keyPrefix: "settings.postProcessing.api",
      keywords: ["openai", "provider", "api key", "model"],
    },
    {
      target: {
        section: "settings",
        unifiedSection: "features",
        featurePanel: "aiConfig",
      },
      keyPrefix: "settings.postProcessing.prompts",
      keywords: ["template", "instructions", "refine"],
    },
  ],
  suggestions: [
    {
      target: {
        section: "settings",
        unifiedSection: "features",
        featurePanel: "suggestions",
      },
      keyPrefix: "settings.suggestions.enable",
      keywords: [
        "coaching",
        "quick responses",
        "rag suggestions",
        "talking points",
      ],
    },
    {
      target: {
        section: "settings",
        unifiedSection: "features",
        featurePanel: "suggestions",
      },
      keyPrefix: "settings.suggestions.add",
      keywords: ["templates", "trigger phrases", "response snippets"],
    },
  ],
  history: [
    {
      target: { section: "history" },
      keyPrefix: "settings.history",
      keywords: ["transcriptions", "recordings", "export"],
    },
    {
      target: {
        section: "settings",
        unifiedSection: "features",
        featurePanel: "batchProcessing",
      },
      keyPrefix: "batchProcessing",
      keywords: ["batch", "queue", "import", "transcription files"],
    },
  ],
  debug: [
    {
      target: { section: "debug" },
      keyPrefix: "settings.debug.logLevel",
      keywords: ["verbosity", "logging"],
    },
    {
      target: { section: "debug" },
      keyPrefix: "settings.debug.updateChecks",
      keywords: ["version", "updates"],
    },
    {
      target: { section: "debug" },
      keyPrefix: "settings.debug.historyLimit",
      keywords: ["entries", "maximum"],
    },
    {
      target: { section: "debug" },
      keyPrefix: "settings.debug.recordingRetention",
      keywords: ["auto-delete", "storage", "space"],
    },
  ],
  about: [
    {
      target: { section: "about" },
      keyPrefix: "settings.about.version",
      keywords: ["version", "release"],
    },
    {
      target: { section: "about" },
      keyPrefix: "settings.about.sourceCode",
      keywords: ["github", "contribute"],
    },
  ],
};

export function useSettingsSearch(): UseSettingsSearchReturn {
  const { t } = useTranslation();
  const { settings } = useSettings();
  const [query, setQuery] = useState("");
  const [isOpen, setIsOpen] = useState(false);

  const items = useMemo<SearchableItem[]>(() => {
    const result: SearchableItem[] = [];

    for (const [, entries] of Object.entries(SEARCHABLE_ENTRIES)) {
      for (const entry of entries) {
        const sectionConfig = SECTIONS_CONFIG[entry.target.section];
        if (!sectionConfig || !sectionConfig.enabled(settings)) continue;

        const label = t(`${entry.keyPrefix}.title`, {
          defaultValue: t(`${entry.keyPrefix}.label`, { defaultValue: "" }),
        });
        const description = t(`${entry.keyPrefix}.description`, {
          defaultValue: "",
        });

        if (!label) continue;

        result.push({
          id: entry.keyPrefix,
          sectionLabel: t(sectionConfig.labelKey),
          label,
          description,
          keywords: entry.keywords ?? [],
          target: entry.target,
        });
      }
    }

    return result;
  }, [t, settings]);

  const fuse = useMemo(
    () =>
      new Fuse(items, {
        keys: [
          { name: "label", weight: 0.4 },
          { name: "description", weight: 0.3 },
          { name: "keywords", weight: 0.2 },
          { name: "sectionLabel", weight: 0.1 },
        ],
        threshold: 0.4,
        includeMatches: true,
      }),
    [items],
  );

  const results = useMemo<SearchableItem[]>(() => {
    if (!query.trim()) return items;
    return fuse.search(query).map((r) => r.item);
  }, [query, fuse, items]);

  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === "k") {
        e.preventDefault();
        setIsOpen((prev) => !prev);
      }
    },
    [setIsOpen],
  );

  useEffect(() => {
    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [handleKeyDown]);

  // Reset query when closing
  useEffect(() => {
    if (!isOpen) setQuery("");
  }, [isOpen]);

  return { results, query, setQuery, isOpen, setIsOpen };
}
