import React, { useState, useEffect, useRef, useCallback } from "react";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useTranslation } from "react-i18next";
import { syncLanguageFromSettings } from "@/i18n";
import "./PresentationMode.css";

interface ActiveListeningSegmentEvent {
  session_id: string;
  transcription: string;
  timestamp: number;
  speaker_id: number | null;
  speaker_label: string | null;
}

interface ActiveListeningInsightEvent {
  session_id: string;
  chunk: string;
  done: boolean;
}

interface ActiveListeningStateEvent {
  state: "idle" | "listening" | "processing" | "error";
  session_id: string | null;
  error: string | null;
}

interface TranscriptLine {
  id: string;
  text: string;
  speakerLabel: string | null;
  speakerId: number | null;
  timestamp: number;
  insight: string;
  isInsightStreaming: boolean;
}

const PresentationMode: React.FC = () => {
  const { t } = useTranslation();
  const [lines, setLines] = useState<TranscriptLine[]>([]);
  const [isActive, setIsActive] = useState(false);
  const [fontSize, setFontSize] = useState(32);
  const [showInsights, setShowInsights] = useState(true);
  const scrollRef = useRef<HTMLDivElement>(null);

  // Sync language on mount
  useEffect(() => {
    syncLanguageFromSettings();
  }, []);

  // Auto-scroll to bottom
  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [lines]);

  // Keyboard controls
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        getCurrentWindow().close();
      } else if (e.key === "+" || e.key === "=") {
        setFontSize((prev) => Math.min(prev + 4, 72));
      } else if (e.key === "-") {
        setFontSize((prev) => Math.max(prev - 4, 16));
      } else if (e.key === "i" || e.key === "I") {
        setShowInsights((prev) => !prev);
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, []);

  // Listen for Active Listening events
  useEffect(() => {
    const unlistenFns: UnlistenFn[] = [];
    let isMounted = true;

    const setupListeners = async () => {
      // State changes
      const unlistenState = await listen<ActiveListeningStateEvent>(
        "active-listening-state-changed",
        (event) => {
          if (!isMounted) return;
          const payload = event.payload;
          if (payload.state === "listening") {
            setIsActive(true);
            if (payload.session_id) {
              setLines([]);
            }
          } else if (payload.state === "idle") {
            setIsActive(false);
          }
        },
      );
      unlistenFns.push(unlistenState);

      // Segments (transcription text)
      const unlistenSegment = await listen<ActiveListeningSegmentEvent>(
        "active-listening-segment",
        (event) => {
          if (!isMounted) return;
          const payload = event.payload;
          const lineId = `${payload.session_id}-${payload.timestamp}`;

          setLines((prev) => {
            if (prev.some((l) => l.id === lineId)) return prev;
            return [
              ...prev,
              {
                id: lineId,
                text: payload.transcription,
                speakerLabel: payload.speaker_label,
                speakerId: payload.speaker_id,
                timestamp: payload.timestamp,
                insight: "",
                isInsightStreaming: true,
              },
            ];
          });
        },
      );
      unlistenFns.push(unlistenSegment);

      // Insights (AI-generated)
      const unlistenInsight = await listen<ActiveListeningInsightEvent>(
        "active-listening-insight",
        (event) => {
          if (!isMounted) return;
          const payload = event.payload;
          setLines((prev) => {
            const lastStreamingIdx = prev.findIndex(
              (l) => l.isInsightStreaming,
            );
            if (lastStreamingIdx === -1) return prev;

            const updated = [...prev];
            if (payload.done) {
              updated[lastStreamingIdx] = {
                ...updated[lastStreamingIdx],
                isInsightStreaming: false,
              };
            } else {
              updated[lastStreamingIdx] = {
                ...updated[lastStreamingIdx],
                insight:
                  updated[lastStreamingIdx].insight + payload.chunk,
              };
            }
            return updated;
          });
        },
      );
      unlistenFns.push(unlistenInsight);
    };

    setupListeners();

    return () => {
      isMounted = false;
      unlistenFns.forEach((fn) => fn());
    };
  }, []);

  const handleClose = useCallback(() => {
    getCurrentWindow().close();
  }, []);

  return (
    <div className="presentation-container">
      {/* Top bar with controls */}
      <div className="presentation-topbar">
        <div className="presentation-topbar-left">
          <span className="presentation-title">
            {t("presentationMode.title", "Presentation Mode")}
          </span>
          {isActive && <span className="presentation-live-dot" />}
        </div>
        <div className="presentation-topbar-right">
          <span className="presentation-hint">
            {t("presentationMode.controls", "Esc: close | +/-: font size | I: toggle insights")}
          </span>
          <button
            className="presentation-close-btn"
            onClick={handleClose}
            aria-label={t("common.close", "Close")}
          >
            &times;
          </button>
        </div>
      </div>

      {/* Main transcript area */}
      <div className="presentation-content" ref={scrollRef}>
        {lines.length === 0 ? (
          <div className="presentation-empty">
            <p style={{ fontSize: `${fontSize}px` }}>
              {isActive
                ? t("presentationMode.listening", "Listening for speech...")
                : t("presentationMode.waiting", "Start Active Listening to begin")}
            </p>
          </div>
        ) : (
          <div className="presentation-lines">
            {lines.map((line) => (
              <div key={line.id} className="presentation-line">
                {line.speakerLabel && (
                  <span
                    className={`presentation-speaker speaker-${line.speakerId ?? 0}`}
                    style={{ fontSize: `${fontSize * 0.6}px` }}
                  >
                    {line.speakerLabel}
                  </span>
                )}
                <p
                  className="presentation-text"
                  style={{ fontSize: `${fontSize}px` }}
                >
                  {line.text}
                </p>
                {showInsights && line.insight && (
                  <p
                    className="presentation-insight"
                    style={{ fontSize: `${fontSize * 0.65}px` }}
                  >
                    {line.insight}
                    {line.isInsightStreaming && (
                      <span className="typing-cursor" />
                    )}
                  </p>
                )}
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
};

export default PresentationMode;
