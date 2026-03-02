import { useMemo, useEffect, useState, useCallback } from "react";
import Fuse from "fuse.js";
import { useTranslation } from "react-i18next";
import { SECTIONS_CONFIG, type SidebarSection } from "@/components/Sidebar";
import { useSettings } from "@/hooks/useSettings";

export interface SearchableItem {
  id: string;
  section: SidebarSection;
  sectionLabel: string;
  label: string;
  description: string;
  keywords: string[];
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
 * - labs: Active Listening, Ask AI, Knowledge Base, Post-Processing
 * - history: Transcription history
 * - about: Version, source code
 * - debug: Log level, updates (only visible in debug mode)
 */
const SEARCHABLE_ENTRIES: Record<
  string,
  { section: SidebarSection; keyPrefix: string; keywords?: string[] }[]
> = {
  home: [
    {
      section: "home",
      keyPrefix: "settings.general.shortcut",
      keywords: ["hotkey", "keybind", "keyboard"],
    },
    {
      section: "home",
      keyPrefix: "settings.general.language",
      keywords: ["locale", "speech recognition"],
    },
    {
      section: "home",
      keyPrefix: "settings.general.pushToTalk",
      keywords: ["hold", "record"],
    },
  ],
  settings: [
    {
      section: "settings",
      keyPrefix: "settings.sound.microphone",
      keywords: ["mic", "input device", "audio input"],
    },
    {
      section: "settings",
      keyPrefix: "settings.sound.audioFeedback",
      keywords: ["sound", "beep", "notification"],
    },
    {
      section: "settings",
      keyPrefix: "settings.sound.outputDevice",
      keywords: ["speaker", "audio output"],
    },
    {
      section: "settings",
      keyPrefix: "settings.sound.volume",
      keywords: ["loudness", "level"],
    },
    {
      section: "settings",
      keyPrefix: "settings.advanced.startHidden",
      keywords: ["tray", "minimize", "launch"],
    },
    {
      section: "settings",
      keyPrefix: "settings.advanced.autostart",
      keywords: ["login", "boot", "startup"],
    },
    {
      section: "settings",
      keyPrefix: "settings.advanced.overlay",
      keywords: ["position", "visual", "feedback"],
    },
    {
      section: "settings",
      keyPrefix: "settings.advanced.pasteMethod",
      keywords: ["clipboard", "typing", "insert"],
    },
    {
      section: "settings",
      keyPrefix: "settings.advanced.clipboardHandling",
      keywords: ["copy", "paste"],
    },
    {
      section: "settings",
      keyPrefix: "settings.advanced.translateToEnglish",
      keywords: ["translation", "language"],
    },
    {
      section: "settings",
      keyPrefix: "settings.advanced.modelUnload",
      keywords: ["memory", "gpu", "unload"],
    },
    {
      section: "settings",
      keyPrefix: "settings.advanced.customWords",
      keywords: ["dictionary", "correction", "spelling", "vocabulary"],
    },
  ],
  labs: [
    {
      section: "labs",
      keyPrefix: "labs.activeListening",
      keywords: ["active listening", "ollama", "continuous", "llm"],
    },
    {
      section: "labs",
      keyPrefix: "labs.askAi",
      keywords: ["ask ai", "conversation", "voice chat", "llm"],
    },
    {
      section: "labs",
      keyPrefix: "labs.knowledgeBase",
      keywords: ["rag", "documents", "embeddings", "search"],
    },
    {
      section: "labs",
      keyPrefix: "labs.postProcessing",
      keywords: ["post-processing", "cleanup", "formatting", "api"],
    },
    {
      section: "labs",
      keyPrefix: "settings.postProcessing.api",
      keywords: ["openai", "provider", "api key", "model"],
    },
    {
      section: "labs",
      keyPrefix: "settings.postProcessing.prompts",
      keywords: ["template", "instructions", "refine"],
    },
  ],
  history: [
    {
      section: "history",
      keyPrefix: "settings.history",
      keywords: ["transcriptions", "recordings", "export"],
    },
  ],
  debug: [
    {
      section: "debug",
      keyPrefix: "settings.debug.logLevel",
      keywords: ["verbosity", "logging"],
    },
    {
      section: "debug",
      keyPrefix: "settings.debug.updateChecks",
      keywords: ["version", "updates"],
    },
    {
      section: "debug",
      keyPrefix: "settings.debug.historyLimit",
      keywords: ["entries", "maximum"],
    },
    {
      section: "debug",
      keyPrefix: "settings.debug.recordingRetention",
      keywords: ["auto-delete", "storage", "space"],
    },
  ],
  about: [
    {
      section: "about",
      keyPrefix: "settings.about.version",
      keywords: ["version", "release"],
    },
    {
      section: "about",
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
        const sectionConfig = SECTIONS_CONFIG[entry.section];
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
          section: entry.section,
          sectionLabel: t(sectionConfig.labelKey),
          label,
          description,
          keywords: entry.keywords ?? [],
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
