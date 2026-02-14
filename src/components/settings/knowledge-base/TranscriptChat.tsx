import React, { useState, useRef, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { Send, Trash2 } from "lucide-react";
import { listen } from "@tauri-apps/api/event";
import { commands, type RagStats } from "@/bindings";

interface ChatMessage {
  role: "user" | "assistant";
  content: string;
}

export const TranscriptChat: React.FC = () => {
  const { t } = useTranslation();
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [input, setInput] = useState("");
  const [isGenerating, setIsGenerating] = useState(false);
  const [stats, setStats] = useState<RagStats | null>(null);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLTextAreaElement>(null);

  // Check KB stats on mount
  useEffect(() => {
    const fetchStats = async () => {
      try {
        const result = await commands.ragGetStats();
        if (result.status === "ok") {
          setStats(result.data);
        }
      } catch {
        // Ignore - KB may not be enabled
      }
    };
    fetchStats();
  }, []);

  // Auto-scroll to bottom on new messages
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  // Listen for streaming chunks
  useEffect(() => {
    let currentResponse = "";

    const setupListeners = async () => {
      const unlistenChunk = await listen<string>("rag-chat-chunk", (event) => {
        currentResponse += event.payload;
        setMessages((prev) => {
          const updated = [...prev];
          const lastMsg = updated[updated.length - 1];
          if (lastMsg && lastMsg.role === "assistant") {
            updated[updated.length - 1] = {
              ...lastMsg,
              content: currentResponse,
            };
          }
          return updated;
        });
      });

      const unlistenComplete = await listen("rag-chat-complete", () => {
        currentResponse = "";
        setIsGenerating(false);
      });

      return () => {
        unlistenChunk();
        unlistenComplete();
      };
    };

    const cleanupPromise = setupListeners();
    return () => {
      cleanupPromise.then((cleanup) => cleanup());
    };
  }, []);

  const buildConversationContext = useCallback(() => {
    if (messages.length === 0) return undefined;
    return messages
      .map((m) => `${m.role === "user" ? "User" : "Assistant"}: ${m.content}`)
      .join("\n\n");
  }, [messages]);

  const handleSend = useCallback(async () => {
    const question = input.trim();
    if (!question || isGenerating) return;

    setInput("");
    setIsGenerating(true);

    // Add user message
    setMessages((prev) => [...prev, { role: "user", content: question }]);

    // Add empty assistant message for streaming
    setMessages((prev) => [...prev, { role: "assistant", content: "" }]);

    try {
      const context = buildConversationContext();
      const result = await commands.ragChat(question, context ?? null);

      if (result.status !== "ok") {
        setMessages((prev) => {
          const updated = [...prev];
          updated[updated.length - 1] = {
            role: "assistant",
            content: `Error: ${result.error}`,
          };
          return updated;
        });
        setIsGenerating(false);
      }
      // Success case is handled by the event listener setting the final content
    } catch (error) {
      setMessages((prev) => {
        const updated = [...prev];
        updated[updated.length - 1] = {
          role: "assistant",
          content: `Error: ${error}`,
        };
        return updated;
      });
      setIsGenerating(false);
    }
  }, [input, isGenerating, buildConversationContext]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === "Enter" && !e.shiftKey) {
        e.preventDefault();
        handleSend();
      }
    },
    [handleSend],
  );

  const clearChat = useCallback(() => {
    setMessages([]);
    setInput("");
  }, []);

  const hasDocuments = stats && stats.document_count > 0;

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between">
        <h3 className="text-xs font-medium text-mid-gray uppercase tracking-wide">
          {t("settings.knowledgeBase.chat.title")}
        </h3>
        {messages.length > 0 && (
          <button
            onClick={clearChat}
            className="text-text/50 hover:text-primary-light transition-colors cursor-pointer p-1"
            title={t("settings.knowledgeBase.chat.clearChat")}
          >
            <Trash2 width={14} height={14} />
          </button>
        )}
      </div>

      {!hasDocuments ? (
        <div className="p-4 bg-mid-gray/5 rounded-lg border border-mid-gray/20">
          <p className="text-sm text-mid-gray">
            {t("settings.knowledgeBase.chat.noDocuments")}
          </p>
        </div>
      ) : (
        <>
          {/* Chat messages */}
          <div className="bg-background border border-mid-gray/20 rounded-lg max-h-[400px] min-h-[200px] overflow-y-auto">
            {messages.length === 0 ? (
              <div className="flex items-center justify-center h-[200px] text-text/40 text-sm">
                {t("settings.knowledgeBase.chat.description")}
              </div>
            ) : (
              <div className="p-3 space-y-3">
                {messages.map((msg, idx) => (
                  <div
                    key={idx}
                    className={`flex flex-col gap-1 ${
                      msg.role === "user" ? "items-end" : "items-start"
                    }`}
                  >
                    <span className="text-xs text-mid-gray">
                      {msg.role === "user"
                        ? t("settings.knowledgeBase.chat.you")
                        : t("settings.knowledgeBase.chat.ai")}
                    </span>
                    <div
                      className={`rounded-lg px-3 py-2 text-sm max-w-[85%] whitespace-pre-wrap ${
                        msg.role === "user"
                          ? "bg-primary-light/20 text-text"
                          : "bg-mid-gray/10 text-text"
                      }`}
                    >
                      {msg.content ||
                        (isGenerating
                          ? t("settings.knowledgeBase.chat.thinking")
                          : "")}
                    </div>
                  </div>
                ))}
                <div ref={messagesEndRef} />
              </div>
            )}
          </div>

          {/* Input area */}
          <div className="flex gap-2">
            <textarea
              ref={inputRef}
              value={input}
              onChange={(e) => setInput(e.target.value)}
              onKeyDown={handleKeyDown}
              placeholder={t("settings.knowledgeBase.chat.placeholder")}
              disabled={isGenerating}
              rows={1}
              className="flex-1 px-3 py-2 text-sm bg-mid-gray/10 border border-mid-gray/20 rounded-lg resize-none focus:outline-none focus:border-primary-light disabled:opacity-50"
            />
            <button
              onClick={handleSend}
              disabled={isGenerating || !input.trim()}
              className="px-3 py-2 bg-primary-light/20 border border-primary-light/50 rounded-lg hover:bg-primary-light/30 transition-colors cursor-pointer disabled:opacity-50 disabled:cursor-not-allowed"
              title={t("settings.knowledgeBase.chat.send")}
            >
              <Send width={16} height={16} />
            </button>
          </div>
        </>
      )}
    </div>
  );
};
