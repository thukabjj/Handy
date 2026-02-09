import React, { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Clock, Trash2, Download, ChevronDown, ChevronUp, MessageSquare } from "lucide-react";
import { commands, HistoryEntry } from "@/bindings";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { SettingsGroup } from "@/components/ui";
import { Button } from "../../ui/Button";

interface GroupedSession {
  date: string;
  entries: HistoryEntry[];
  totalDuration: string;
}

// Helper to format timestamp
const formatDate = (timestamp: number): string => {
  const date = new Date(timestamp);
  return date.toLocaleDateString(undefined, {
    weekday: "short",
    year: "numeric",
    month: "short",
    day: "numeric",
  });
};

const formatTime = (timestamp: number): string => {
  const date = new Date(timestamp);
  return date.toLocaleTimeString(undefined, {
    hour: "2-digit",
    minute: "2-digit",
  });
};

// Group entries by date
const groupEntriesByDate = (entries: HistoryEntry[]): GroupedSession[] => {
  const groups = new Map<string, HistoryEntry[]>();

  entries.forEach((entry) => {
    const dateKey = formatDate(entry.timestamp);
    const existing = groups.get(dateKey) || [];
    groups.set(dateKey, [...existing, entry]);
  });

  return Array.from(groups.entries()).map(([date, groupEntries]) => ({
    date,
    entries: groupEntries.sort((a, b) => b.timestamp - a.timestamp),
    totalDuration: `${groupEntries.length} insights`,
  }));
};

// Single insight entry
const InsightEntry: React.FC<{
  entry: HistoryEntry;
  onDelete: (id: number) => void;
}> = ({ entry, onDelete }) => {
  const { t } = useTranslation();
  const [isExpanded, setIsExpanded] = useState(false);

  return (
    <div className="border border-mid-gray/20 rounded-lg overflow-hidden bg-white/50 dark:bg-black/20">
      <button
        onClick={() => setIsExpanded(!isExpanded)}
        className="w-full p-3 flex items-start gap-3 hover:bg-mid-gray/5 transition-colors text-left"
      >
        <div className="flex-shrink-0 mt-0.5">
          <MessageSquare className="h-4 w-4 text-green-500" />
        </div>
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 mb-1">
            <span className="text-xs text-mid-gray">
              {formatTime(entry.timestamp)}
            </span>
          </div>
          <p className="text-sm text-text line-clamp-2">
            {entry.transcription_text}
          </p>
        </div>
        <div className="flex-shrink-0">
          {isExpanded ? (
            <ChevronUp className="h-4 w-4 text-mid-gray" />
          ) : (
            <ChevronDown className="h-4 w-4 text-mid-gray" />
          )}
        </div>
      </button>

      {isExpanded && (
        <div className="px-3 pb-3 border-t border-mid-gray/10">
          <div className="mt-3 space-y-3">
            <div>
              <p className="text-xs font-semibold text-mid-gray uppercase tracking-wide mb-1">
                {t("sessionViewer.transcription", "Transcription")}
              </p>
              <p className="text-sm text-text bg-mid-gray/5 p-2 rounded">
                {entry.transcription_text}
              </p>
            </div>
            {entry.post_processed_text && (
              <div>
                <p className="text-xs font-semibold text-green-600 uppercase tracking-wide mb-1">
                  {t("sessionViewer.insight", "AI Insight")}
                </p>
                <p className="text-sm text-text bg-green-50 dark:bg-green-900/20 p-2 rounded border-l-2 border-green-500">
                  {entry.post_processed_text}
                </p>
              </div>
            )}
            <div className="flex gap-2 pt-2">
              <Button
                variant="ghost"
                size="sm"
                onClick={(e) => {
                  e.stopPropagation();
                  onDelete(entry.id);
                }}
                className="text-red-500 hover:text-red-600 hover:bg-red-50"
              >
                <Trash2 className="h-3 w-3 mr-1" />
                {t("sessionViewer.delete", "Delete")}
              </Button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

// Session group (entries for a single day)
const SessionGroup: React.FC<{
  group: GroupedSession;
  onDeleteEntry: (id: number) => void;
  onExportGroup: (entries: HistoryEntry[]) => void;
}> = ({ group, onDeleteEntry, onExportGroup }) => {
  const { t } = useTranslation();
  const [isExpanded, setIsExpanded] = useState(true);

  return (
    <div className="border border-mid-gray/20 rounded-lg overflow-hidden">
      <button
        onClick={() => setIsExpanded(!isExpanded)}
        className="w-full p-4 flex items-center gap-3 bg-mid-gray/5 hover:bg-mid-gray/10 transition-colors"
      >
        <div className="flex-1 text-left">
          <h3 className="font-semibold text-text">{group.date}</h3>
          <div className="flex items-center gap-3 mt-1 text-xs text-mid-gray">
            <span className="flex items-center gap-1">
              <Clock className="h-3 w-3" />
              {group.totalDuration}
            </span>
          </div>
        </div>
        <div className="flex items-center gap-2">
          <Button
            variant="ghost"
            size="sm"
            onClick={(e) => {
              e.stopPropagation();
              onExportGroup(group.entries);
            }}
            className="text-mid-gray hover:text-text"
          >
            <Download className="h-4 w-4" />
          </Button>
          {isExpanded ? (
            <ChevronUp className="h-5 w-5 text-mid-gray" />
          ) : (
            <ChevronDown className="h-5 w-5 text-mid-gray" />
          )}
        </div>
      </button>

      {isExpanded && (
        <div className="p-3 space-y-2">
          {group.entries.map((entry) => (
            <InsightEntry
              key={entry.id}
              entry={entry}
              onDelete={onDeleteEntry}
            />
          ))}
        </div>
      )}
    </div>
  );
};

export const SessionViewer: React.FC = () => {
  const { t } = useTranslation();
  const [entries, setEntries] = useState<HistoryEntry[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const loadEntries = async () => {
    setIsLoading(true);
    setError(null);
    try {
      const result = await commands.getHistoryEntries();
      if (result.status === "ok") {
        // Filter to only show entries with post-processed text (Active Listening insights)
        const activeListeningEntries = result.data.filter(
          (e) => e.post_processed_text && e.post_processed_text.length > 0
        );
        setEntries(activeListeningEntries);
      } else {
        setError(result.error);
      }
    } catch (err) {
      setError(String(err));
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    loadEntries();
  }, []);

  const handleDeleteEntry = async (id: number) => {
    try {
      const result = await commands.deleteHistoryEntry(id);
      if (result.status === "ok") {
        setEntries((prev) => prev.filter((e) => e.id !== id));
      }
    } catch (err) {
      console.error("Failed to delete entry:", err);
    }
  };

  const handleExportGroup = async (groupEntries: HistoryEntry[]) => {
    const markdown = groupEntries
      .map((entry) => {
        const time = formatTime(entry.timestamp);
        let text = `## ${time}\n\n`;
        text += `**Transcription:**\n${entry.transcription_text}\n\n`;
        if (entry.post_processed_text) {
          text += `**AI Insight:**\n${entry.post_processed_text}\n\n`;
        }
        return text;
      })
      .join("---\n\n");

    const header = `# Active Listening Session - ${formatDate(groupEntries[0]?.timestamp || Date.now())}\n\n`;
    await writeText(header + markdown);
  };

  const groupedSessions = groupEntriesByDate(entries);

  if (isLoading) {
    return (
      <SettingsGroup title={t("sessionViewer.title", "Session History")}>
        <div className="p-4 text-center text-mid-gray">
          {t("sessionViewer.loading", "Loading sessions...")}
        </div>
      </SettingsGroup>
    );
  }

  if (error) {
    return (
      <SettingsGroup title={t("sessionViewer.title", "Session History")}>
        <div className="p-4 text-center text-red-500">
          {t("sessionViewer.error", "Failed to load sessions: {{error}}", {
            error,
          })}
        </div>
      </SettingsGroup>
    );
  }

  if (entries.length === 0) {
    return (
      <SettingsGroup title={t("sessionViewer.title", "Session History")}>
        <div className="p-6 text-center">
          <MessageSquare className="h-12 w-12 mx-auto text-mid-gray/50 mb-3" />
          <p className="text-mid-gray">
            {t(
              "sessionViewer.empty",
              "No Active Listening sessions yet. Start a session to see your insights here."
            )}
          </p>
        </div>
      </SettingsGroup>
    );
  }

  return (
    <SettingsGroup title={t("sessionViewer.title", "Session History")}>
      <div className="space-y-3">
        <p className="text-sm text-mid-gray">
          {t("sessionViewer.description", "View and export your past Active Listening sessions and AI-generated insights.")}
        </p>
        {groupedSessions.map((group) => (
          <SessionGroup
            key={group.date}
            group={group}
            onDeleteEntry={handleDeleteEntry}
            onExportGroup={handleExportGroup}
          />
        ))}
      </div>
    </SettingsGroup>
  );
};
