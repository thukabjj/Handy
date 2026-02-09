import React, { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  Clock,
  Trash2,
  Download,
  ChevronDown,
  ChevronUp,
  MessageSquare,
  Search,
  RefreshCcw,
} from "lucide-react";
import { commands, AskAiConversation, ConversationTurn } from "@/bindings";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { SettingsGroup } from "@/components/ui";
import { Button } from "../../ui/Button";
import { Input } from "../../ui/Input";

// Helper to format timestamp
const formatDate = (timestamp: number): string => {
  const date = new Date(timestamp);
  return date.toLocaleDateString(undefined, {
    weekday: "short",
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

// Format conversation to markdown for export
const formatConversationAsMarkdown = (conversation: AskAiConversation): string => {
  const header = `# ${conversation.title || "Ask AI Conversation"}\n`;
  const date = `_${formatDate(conversation.created_at)} at ${formatTime(conversation.created_at)}_\n\n`;

  const turns = conversation.turns
    .map((turn, index) => {
      return `## Turn ${index + 1}\n\n**Q:** ${turn.question}\n\n**A:** ${turn.response}\n`;
    })
    .join("\n---\n\n");

  return header + date + turns;
};

// Single turn display
const TurnDisplay: React.FC<{ turn: ConversationTurn; index: number }> = ({
  turn,
  index,
}) => {
  const { t } = useTranslation();

  return (
    <div className="space-y-2">
      <div className="flex items-start gap-2">
        <span className="text-xs font-bold text-purple-600 flex-shrink-0 pt-0.5">
          {t("askAi.conversation.question", "Q:")}
        </span>
        <p className="text-sm text-text">{turn.question}</p>
      </div>
      <div className="flex items-start gap-2">
        <span className="text-xs font-bold text-green-600 flex-shrink-0 pt-0.5">
          {t("askAi.conversation.answer", "A:")}
        </span>
        <p className="text-sm text-text whitespace-pre-wrap">{turn.response}</p>
      </div>
    </div>
  );
};

// Conversation card
const ConversationCard: React.FC<{
  conversation: AskAiConversation;
  onDelete: (id: string) => void;
  onExport: (conversation: AskAiConversation) => void;
}> = ({ conversation, onDelete, onExport }) => {
  const { t } = useTranslation();
  const [isExpanded, setIsExpanded] = useState(false);

  const title =
    conversation.title ||
    conversation.turns[0]?.question.slice(0, 50) + "..." ||
    t("askAi.conversation.untitled", "Untitled conversation");

  return (
    <div className="border border-mid-gray/20 rounded-lg overflow-hidden">
      <button
        onClick={() => setIsExpanded(!isExpanded)}
        className="w-full p-4 flex items-start gap-3 hover:bg-mid-gray/5 transition-colors text-left"
      >
        <div className="flex-shrink-0 mt-0.5">
          <MessageSquare className="h-5 w-5 text-purple-500" />
        </div>
        <div className="flex-1 min-w-0">
          <h3 className="font-semibold text-text line-clamp-1">{title}</h3>
          <div className="flex items-center gap-3 mt-1 text-xs text-mid-gray">
            <span className="flex items-center gap-1">
              <Clock className="h-3 w-3" />
              {formatDate(conversation.created_at)}
            </span>
            <span>
              {t("askAi.conversation.turns", "{{count}} turns", {
                count: conversation.turns.length,
              })}
            </span>
          </div>
        </div>
        <div className="flex items-center gap-1 flex-shrink-0">
          <Button
            variant="ghost"
            size="sm"
            onClick={(e) => {
              e.stopPropagation();
              onExport(conversation);
            }}
            className="text-mid-gray hover:text-text p-1"
          >
            <Download className="h-4 w-4" />
          </Button>
          <Button
            variant="ghost"
            size="sm"
            onClick={(e) => {
              e.stopPropagation();
              onDelete(conversation.id);
            }}
            className="text-red-400 hover:text-red-600 hover:bg-red-50 p-1"
          >
            <Trash2 className="h-4 w-4" />
          </Button>
          {isExpanded ? (
            <ChevronUp className="h-5 w-5 text-mid-gray ml-1" />
          ) : (
            <ChevronDown className="h-5 w-5 text-mid-gray ml-1" />
          )}
        </div>
      </button>

      {isExpanded && (
        <div className="px-4 pb-4 border-t border-mid-gray/10">
          <div className="mt-4 space-y-4">
            {conversation.turns.map((turn, index) => (
              <div
                key={turn.id}
                className={
                  index > 0
                    ? "pt-4 border-t border-mid-gray/10"
                    : ""
                }
              >
                <TurnDisplay turn={turn} index={index} />
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};

export const ConversationHistory: React.FC = () => {
  const { t } = useTranslation();
  const [conversations, setConversations] = useState<AskAiConversation[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState("");

  const loadConversations = async () => {
    setIsLoading(true);
    setError(null);
    try {
      const result = await commands.listAskAiConversations(100);
      if (result.status === "ok") {
        setConversations(result.data);
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
    loadConversations();
  }, []);

  const handleDelete = async (id: string) => {
    try {
      const result = await commands.deleteAskAiConversationFromHistory(id);
      if (result.status === "ok") {
        setConversations((prev) => prev.filter((c) => c.id !== id));
      }
    } catch (err) {
      console.error("Failed to delete conversation:", err);
    }
  };

  const handleExport = async (conversation: AskAiConversation) => {
    const markdown = formatConversationAsMarkdown(conversation);
    await writeText(markdown);
  };

  // Filter conversations by search query
  const filteredConversations = searchQuery
    ? conversations.filter((c) => {
        const searchLower = searchQuery.toLowerCase();
        if (c.title?.toLowerCase().includes(searchLower)) return true;
        return c.turns.some(
          (t) =>
            t.question.toLowerCase().includes(searchLower) ||
            t.response.toLowerCase().includes(searchLower)
        );
      })
    : conversations;

  if (isLoading) {
    return (
      <SettingsGroup title={t("askAi.history.title", "Conversation History")}>
        <div className="p-4 text-center text-mid-gray">
          {t("askAi.history.loading", "Loading conversations...")}
        </div>
      </SettingsGroup>
    );
  }

  if (error) {
    return (
      <SettingsGroup title={t("askAi.history.title", "Conversation History")}>
        <div className="p-4 text-center text-red-500">
          {t("askAi.history.error", "Failed to load conversations: {{error}}", {
            error,
          })}
        </div>
      </SettingsGroup>
    );
  }

  return (
    <SettingsGroup title={t("askAi.history.title", "Conversation History")}>
      <div className="space-y-3">
        {/* Search and refresh */}
        <div className="flex items-center gap-2">
          <div className="relative flex-1">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-mid-gray" />
            <Input
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder={t("askAi.history.search", "Search conversations...")}
              className="pl-9"
            />
          </div>
          <Button
            variant="ghost"
            size="md"
            onClick={loadConversations}
            className="flex-shrink-0"
          >
            <RefreshCcw className="h-4 w-4" />
          </Button>
        </div>

        {/* Conversation list */}
        {conversations.length === 0 ? (
          <div className="p-6 text-center">
            <MessageSquare className="h-12 w-12 mx-auto text-mid-gray/50 mb-3" />
            <p className="text-mid-gray">
              {t(
                "askAi.history.empty",
                "No conversations yet. Use Ask AI to start a conversation."
              )}
            </p>
          </div>
        ) : filteredConversations.length === 0 ? (
          <div className="p-4 text-center text-mid-gray">
            {t("askAi.history.noResults", "No conversations match your search.")}
          </div>
        ) : (
          <div className="space-y-2">
            {filteredConversations.map((conversation) => (
              <ConversationCard
                key={conversation.id}
                conversation={conversation}
                onDelete={handleDelete}
                onExport={handleExport}
              />
            ))}
          </div>
        )}
      </div>
    </SettingsGroup>
  );
};
