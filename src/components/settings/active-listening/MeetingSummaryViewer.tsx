import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import {
  FileText,
  Download,
  Copy,
  Check,
  ChevronDown,
  ChevronUp,
  ClipboardList,
  Users,
  MessageCircle,
  HelpCircle,
  Loader2,
} from "lucide-react";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { commands, ActiveListeningSession } from "@/bindings";
import { Button } from "../../ui/Button";

// Type definitions - these should match the Rust struct definitions
// These will be available from bindings once regenerated
interface ActionItem {
  description: string;
  assignee: string | null;
  deadline: string | null;
}

interface MeetingSummary {
  session_id: string;
  executive_summary: string;
  decisions: string[];
  action_items: ActionItem[];
  topics: string[];
  follow_ups: string[];
  duration_minutes: number;
  generated_at: number;
}

interface MeetingSummaryViewerProps {
  session: ActiveListeningSession;
  onClose?: () => void;
}

export const MeetingSummaryViewer: React.FC<MeetingSummaryViewerProps> = ({
  session,
  onClose,
}) => {
  const { t } = useTranslation();
  const [summary, setSummary] = useState<MeetingSummary | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [copiedToClipboard, setCopiedToClipboard] = useState(false);
  const [expandedSections, setExpandedSections] = useState<Set<string>>(
    new Set(["executive", "decisions", "actionItems", "topics", "followUps"])
  );

  const generateSummary = async () => {
    setIsLoading(true);
    setError(null);
    try {
      // Use type assertion since commands may not be in bindings yet
      const commandsAny = commands as unknown as Record<string, unknown>;
      if (typeof commandsAny.generateMeetingSummary !== "function") {
        throw new Error("Meeting summary feature not available. Please rebuild the application.");
      }
      const generateFn = commandsAny.generateMeetingSummary as (
        session: ActiveListeningSession
      ) => Promise<{ status: string; data?: MeetingSummary; error?: string }>;
      const result = await generateFn(session);
      if (result.status === "ok" && result.data) {
        setSummary(result.data);
      } else {
        setError(result.error || "Unknown error");
      }
    } catch (err) {
      setError(String(err));
    } finally {
      setIsLoading(false);
    }
  };

  const exportSummary = async (format: "markdown" | "text" | "json") => {
    if (!summary) return;
    try {
      // Use type assertion since commands may not be in bindings yet
      const commandsAny = commands as unknown as Record<string, unknown>;
      if (typeof commandsAny.exportMeetingSummary !== "function") {
        throw new Error("Export feature not available. Please rebuild the application.");
      }
      const exportFn = commandsAny.exportMeetingSummary as (
        summary: MeetingSummary,
        format: string
      ) => Promise<{ status: string; data?: string; error?: string }>;
      const result = await exportFn(summary, format);
      if (result.status === "ok" && result.data) {
        await writeText(result.data);
        setCopiedToClipboard(true);
        setTimeout(() => setCopiedToClipboard(false), 2000);
      }
    } catch (err) {
      console.error("Failed to export summary:", err);
    }
  };

  const toggleSection = (section: string) => {
    setExpandedSections((prev) => {
      const next = new Set(prev);
      if (next.has(section)) {
        next.delete(section);
      } else {
        next.add(section);
      }
      return next;
    });
  };

  const SectionHeader: React.FC<{
    id: string;
    icon: React.ReactNode;
    title: string;
    count?: number;
  }> = ({ id, icon, title, count }) => (
    <button
      onClick={() => toggleSection(id)}
      className="w-full flex items-center justify-between p-3 bg-mid-gray/5 hover:bg-mid-gray/10 rounded-lg transition-colors"
    >
      <div className="flex items-center gap-2">
        {icon}
        <span className="font-semibold text-text">{title}</span>
        {count !== undefined && (
          <span className="text-xs text-mid-gray bg-mid-gray/20 px-2 py-0.5 rounded-full">
            {count}
          </span>
        )}
      </div>
      {expandedSections.has(id) ? (
        <ChevronUp className="h-4 w-4 text-mid-gray" />
      ) : (
        <ChevronDown className="h-4 w-4 text-mid-gray" />
      )}
    </button>
  );

  // Show generate button if no summary yet
  if (!summary && !isLoading) {
    return (
      <div className="p-6 text-center">
        <FileText className="h-12 w-12 mx-auto text-mid-gray/50 mb-3" />
        <h3 className="text-lg font-semibold text-text mb-2">
          {t("settings.activeListening.summary.title", "Meeting Summary")}
        </h3>
        <p className="text-sm text-mid-gray mb-4">
          {session.insights.length > 0
            ? t(
                "settings.activeListening.summary.generate",
                "Generate a comprehensive summary of this session"
              )
            : t(
                "settings.activeListening.summary.noInsights",
                "This session has no insights to summarize."
              )}
        </p>
        {session.insights.length > 0 && (
          <Button onClick={generateSummary} variant="primary">
            {t("settings.activeListening.summary.generate", "Generate Summary")}
          </Button>
        )}
        {error && <p className="mt-4 text-sm text-red-500">{error}</p>}
      </div>
    );
  }

  // Show loading state
  if (isLoading) {
    return (
      <div className="p-6 text-center">
        <Loader2 className="h-8 w-8 mx-auto text-logo-primary animate-spin mb-3" />
        <p className="text-sm text-mid-gray">
          {t(
            "settings.activeListening.summary.generating",
            "Generating summary..."
          )}
        </p>
      </div>
    );
  }

  // Show error state
  if (error) {
    return (
      <div className="p-6 text-center">
        <p className="text-red-500 mb-4">
          {t("settings.activeListening.summary.error", {
            error,
            defaultValue: `Failed to generate summary: ${error}`,
          })}
        </p>
        <Button onClick={generateSummary} variant="secondary">
          {t("common.retry", "Retry")}
        </Button>
      </div>
    );
  }

  // Show summary
  return (
    <div className="space-y-4">
      {/* Header with export options */}
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-lg font-semibold text-text">
            {t("settings.activeListening.summary.title", "Meeting Summary")}
          </h3>
          <p className="text-sm text-mid-gray">
            {t("settings.activeListening.summary.duration", {
              minutes: summary?.duration_minutes,
              defaultValue: `${summary?.duration_minutes} minutes`,
            })}
          </p>
        </div>
        <div className="flex gap-2">
          <Button
            variant="ghost"
            size="sm"
            onClick={() => exportSummary("markdown")}
            title={t(
              "settings.activeListening.summary.export.markdown",
              "Export as Markdown"
            )}
          >
            <Download className="h-4 w-4" />
          </Button>
          <Button
            variant="ghost"
            size="sm"
            onClick={() => exportSummary("text")}
            title={t(
              "settings.activeListening.summary.export.copy",
              "Copy to Clipboard"
            )}
          >
            {copiedToClipboard ? (
              <Check className="h-4 w-4 text-green-500" />
            ) : (
              <Copy className="h-4 w-4" />
            )}
          </Button>
        </div>
      </div>

      {/* Executive Summary */}
      <div className="space-y-2">
        <SectionHeader
          id="executive"
          icon={<FileText className="h-4 w-4 text-blue-500" />}
          title={t(
            "settings.activeListening.summary.executiveSummary",
            "Executive Summary"
          )}
        />
        {expandedSections.has("executive") && (
          <div className="p-3 bg-blue-50 dark:bg-blue-900/20 rounded-lg border-l-4 border-blue-500">
            <p className="text-sm text-text">
              {summary?.executive_summary ||
                t(
                  "settings.activeListening.summary.noExecutiveSummary",
                  "No summary available"
                )}
            </p>
          </div>
        )}
      </div>

      {/* Key Decisions */}
      <div className="space-y-2">
        <SectionHeader
          id="decisions"
          icon={<ClipboardList className="h-4 w-4 text-purple-500" />}
          title={t("settings.activeListening.summary.decisions", "Key Decisions")}
          count={summary?.decisions.length}
        />
        {expandedSections.has("decisions") && (
          <div className="p-3 space-y-2">
            {summary?.decisions && summary.decisions.length > 0 ? (
              summary.decisions.map((decision, idx) => (
                <div
                  key={idx}
                  className="flex items-start gap-2 p-2 bg-purple-50 dark:bg-purple-900/20 rounded"
                >
                  <span className="flex-shrink-0 w-5 h-5 flex items-center justify-center bg-purple-500 text-white text-xs rounded-full">
                    {idx + 1}
                  </span>
                  <p className="text-sm text-text">{decision}</p>
                </div>
              ))
            ) : (
              <p className="text-sm text-mid-gray italic">
                {t(
                  "settings.activeListening.summary.noDecisions",
                  "No key decisions recorded"
                )}
              </p>
            )}
          </div>
        )}
      </div>

      {/* Action Items */}
      <div className="space-y-2">
        <SectionHeader
          id="actionItems"
          icon={<Users className="h-4 w-4 text-green-500" />}
          title={t(
            "settings.activeListening.summary.actionItems",
            "Action Items"
          )}
          count={summary?.action_items.length}
        />
        {expandedSections.has("actionItems") && (
          <div className="p-3 space-y-2">
            {summary?.action_items && summary.action_items.length > 0 ? (
              summary.action_items.map((item, idx) => (
                <div
                  key={idx}
                  className="flex items-start gap-2 p-2 bg-green-50 dark:bg-green-900/20 rounded border-l-2 border-green-500"
                >
                  <input
                    type="checkbox"
                    className="mt-0.5 h-4 w-4 rounded border-gray-300"
                    disabled
                  />
                  <div className="flex-1">
                    <p className="text-sm text-text">{item.description}</p>
                    <div className="flex gap-3 mt-1">
                      {item.assignee && (
                        <span className="text-xs text-green-600">
                          {t("settings.activeListening.summary.actionItem.assignee", {
                            name: item.assignee,
                            defaultValue: `Assignee: ${item.assignee}`,
                          })}
                        </span>
                      )}
                      {item.deadline && (
                        <span className="text-xs text-orange-600">
                          {t("settings.activeListening.summary.actionItem.deadline", {
                            date: item.deadline,
                            defaultValue: `Due: ${item.deadline}`,
                          })}
                        </span>
                      )}
                    </div>
                  </div>
                </div>
              ))
            ) : (
              <p className="text-sm text-mid-gray italic">
                {t(
                  "settings.activeListening.summary.noActionItems",
                  "No action items identified"
                )}
              </p>
            )}
          </div>
        )}
      </div>

      {/* Topics Discussed */}
      <div className="space-y-2">
        <SectionHeader
          id="topics"
          icon={<MessageCircle className="h-4 w-4 text-orange-500" />}
          title={t(
            "settings.activeListening.summary.topics",
            "Topics Discussed"
          )}
          count={summary?.topics.length}
        />
        {expandedSections.has("topics") && (
          <div className="p-3">
            {summary?.topics && summary.topics.length > 0 ? (
              <div className="flex flex-wrap gap-2">
                {summary.topics.map((topic, idx) => (
                  <span
                    key={idx}
                    className="px-3 py-1 text-sm bg-orange-50 dark:bg-orange-900/20 text-orange-700 dark:text-orange-300 rounded-full"
                  >
                    {topic}
                  </span>
                ))}
              </div>
            ) : (
              <p className="text-sm text-mid-gray italic">
                {t(
                  "settings.activeListening.summary.noTopics",
                  "No specific topics identified"
                )}
              </p>
            )}
          </div>
        )}
      </div>

      {/* Follow-up Questions */}
      <div className="space-y-2">
        <SectionHeader
          id="followUps"
          icon={<HelpCircle className="h-4 w-4 text-cyan-500" />}
          title={t(
            "settings.activeListening.summary.followUps",
            "Follow-up Questions"
          )}
          count={summary?.follow_ups.length}
        />
        {expandedSections.has("followUps") && (
          <div className="p-3 space-y-2">
            {summary?.follow_ups && summary.follow_ups.length > 0 ? (
              summary.follow_ups.map((question, idx) => (
                <div
                  key={idx}
                  className="flex items-start gap-2 p-2 bg-cyan-50 dark:bg-cyan-900/20 rounded"
                >
                  <HelpCircle className="h-4 w-4 text-cyan-500 flex-shrink-0 mt-0.5" />
                  <p className="text-sm text-text">{question}</p>
                </div>
              ))
            ) : (
              <p className="text-sm text-mid-gray italic">
                {t(
                  "settings.activeListening.summary.noFollowUps",
                  "No follow-up questions suggested"
                )}
              </p>
            )}
          </div>
        )}
      </div>

      {/* Close/Back button */}
      {onClose && (
        <div className="pt-4 border-t border-mid-gray/20">
          <Button onClick={onClose} variant="secondary" className="w-full">
            {t("common.close", "Close")}
          </Button>
        </div>
      )}
    </div>
  );
};
