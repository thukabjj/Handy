import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { getCurrentWindow } from "@tauri-apps/api/window";
import React, { useEffect, useRef, useState, useCallback } from "react";
import { useTranslation } from "react-i18next";
import {
  MicrophoneIcon,
  TranscriptionIcon,
  CancelIcon,
} from "../components/icons";
import "./RecordingOverlay.css";
import { commands, AskAiState, AskAiConversation, ConversationTurn } from "@/bindings";
import { syncLanguageFromSettings } from "@/i18n";

type OverlayState =
  | "recording"
  | "transcribing"
  | "active-listening"
  | "active-listening-processing"
  | "ask-ai-recording"
  | "ask-ai-transcribing"
  | "ask-ai-generating"
  | "ask-ai-complete"
  | "ask-ai-error";

// Active listening event types
interface ActiveListeningStateEvent {
  state: "idle" | "listening" | "processing" | "error";
  session_id: string | null;
  error: string | null;
}

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

interface ActiveListeningInsight {
  id: string;
  transcription: string;
  insight: string;
  timestamp: number;
  isStreaming: boolean;
  speakerId: number | null;
  speakerLabel: string | null;
}

// Ask AI event types
interface AskAiStateEvent {
  state: AskAiState;
  question: string | null;
  error: string | null;
  conversation: AskAiConversation | null;
}

interface AskAiResponseEvent {
  chunk: string;
  done: boolean;
}

const RecordingOverlay: React.FC = () => {
  const { t } = useTranslation();
  const [isVisible, setIsVisible] = useState(false);
  const [state, setState] = useState<OverlayState>("recording");
  const [levels, setLevels] = useState<number[]>(Array(16).fill(0));
  const smoothedLevelsRef = useRef<number[]>(Array(16).fill(0));

  // Ask AI response state
  const [askAiQuestion, setAskAiQuestion] = useState<string>("");
  const [askAiResponse, setAskAiResponse] = useState<string>("");
  const [askAiError, setAskAiError] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);
  const responseRef = useRef<HTMLDivElement>(null);
  const autoDismissRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Conversation state for multi-turn
  const [conversation, setConversation] = useState<AskAiConversation | null>(null);

  // Active Listening insight state
  const [activeListeningSessionId, setActiveListeningSessionId] = useState<string | null>(null);
  const [activeListeningInsights, setActiveListeningInsights] = useState<ActiveListeningInsight[]>([]);
  const [currentTranscription, setCurrentTranscription] = useState<string>("");
  const [sessionStartTime, setSessionStartTime] = useState<number | null>(null);
  const insightsScrollRef = useRef<HTMLDivElement>(null);

  // Clear auto-dismiss timer on unmount or state change
  useEffect(() => {
    return () => {
      if (autoDismissRef.current) {
        clearTimeout(autoDismissRef.current);
      }
    };
  }, []);

  // Auto-scroll response
  useEffect(() => {
    if (responseRef.current) {
      responseRef.current.scrollTop = responseRef.current.scrollHeight;
    }
  }, [askAiResponse]);

  // Auto-scroll Active Listening insights
  useEffect(() => {
    if (insightsScrollRef.current) {
      insightsScrollRef.current.scrollTop = insightsScrollRef.current.scrollHeight;
    }
  }, [activeListeningInsights]);

  // Auto-dismiss after completion - disabled for multi-turn conversations
  // Users can now continue asking follow-up questions
  useEffect(() => {
    // Don't auto-dismiss if we have a conversation - user might want to ask follow-ups
    if (state === "ask-ai-error" && !conversation?.turns?.length) {
      // Clear any existing timer
      if (autoDismissRef.current) {
        clearTimeout(autoDismissRef.current);
      }
      // Auto-dismiss errors after 8 seconds only if no conversation
      autoDismissRef.current = setTimeout(() => {
        handleDismiss();
      }, 8000);
    }

    return () => {
      if (autoDismissRef.current) {
        clearTimeout(autoDismissRef.current);
      }
    };
  }, [state, conversation]);

  // Handle window drag
  const handleDragStart = useCallback(async (e: React.MouseEvent) => {
    // Only allow drag from header area
    if ((e.target as HTMLElement).closest('.ask-ai-header-actions')) {
      return; // Don't drag when clicking buttons
    }
    try {
      const window = getCurrentWindow();
      await window.startDragging();
    } catch (err) {
      console.error('Failed to start dragging:', err);
    }
  }, []);

  // Save window bounds when resized or moved
  const saveWindowBounds = useCallback(async () => {
    try {
      const window = getCurrentWindow();
      const size = await window.innerSize();
      const position = await window.innerPosition();
      const scaleFactor = await window.scaleFactor();

      await commands.saveAskAiWindowBounds({
        width: size.width / scaleFactor,
        height: size.height / scaleFactor,
        x: position.x / scaleFactor,
        y: position.y / scaleFactor,
      });
    } catch (err) {
      console.error('Failed to save window bounds:', err);
    }
  }, []);

  // Listen for window resize/move events
  useEffect(() => {
    let debounceTimer: ReturnType<typeof setTimeout> | null = null;
    let unlistenResize: (() => void) | null = null;
    let unlistenMove: (() => void) | null = null;

    const setupListeners = async () => {
      const window = getCurrentWindow();

      unlistenResize = await window.onResized(() => {
        if (debounceTimer) clearTimeout(debounceTimer);
        debounceTimer = setTimeout(saveWindowBounds, 500);
      });

      unlistenMove = await window.onMoved(() => {
        if (debounceTimer) clearTimeout(debounceTimer);
        debounceTimer = setTimeout(saveWindowBounds, 500);
      });
    };

    setupListeners();

    return () => {
      if (debounceTimer) clearTimeout(debounceTimer);
      if (unlistenResize) unlistenResize();
      if (unlistenMove) unlistenMove();
    };
  }, [saveWindowBounds]);

  useEffect(() => {
    const unlistenFns: UnlistenFn[] = [];
    let isMounted = true;

    const setupEventListeners = async () => {
      // Listen for show-overlay event from Rust
      const unlistenShow = await listen("show-overlay", async (event) => {
        if (!isMounted) return;
        // Sync language from settings each time overlay is shown
        await syncLanguageFromSettings();
        const overlayState = event.payload as OverlayState;
        setState(overlayState);
        setIsVisible(true);

        // Reset Ask AI state when starting a new session
        if (
          overlayState === "ask-ai-recording" ||
          overlayState === "ask-ai-generating"
        ) {
          if (overlayState === "ask-ai-generating") {
            // Keep question if transitioning to generating, but reset response
            setAskAiResponse("");
            setAskAiError(null);
            setCopied(false);
          } else {
            // Reset everything for new recording
            setAskAiQuestion("");
            setAskAiResponse("");
            setAskAiError(null);
            setCopied(false);
          }
        }
      });
      unlistenFns.push(unlistenShow);

      // Listen for hide-overlay event from Rust
      const unlistenHide = await listen("hide-overlay", () => {
        if (!isMounted) return;
        setIsVisible(false);
      });
      unlistenFns.push(unlistenHide);

      // Listen for mic-level updates
      const unlistenLevel = await listen<number[]>("mic-level", (event) => {
        if (!isMounted) return;
        const newLevels = event.payload as number[];

        // Apply smoothing to reduce jitter
        const smoothed = smoothedLevelsRef.current.map((prev, i) => {
          const target = newLevels[i] || 0;
          return prev * 0.7 + target * 0.3; // Smooth transition
        });

        smoothedLevelsRef.current = smoothed;
        setLevels(smoothed.slice(0, 9));
      });
      unlistenFns.push(unlistenLevel);

      // Listen for active listening state changes
      const unlistenALState = await listen<ActiveListeningStateEvent>(
        "active-listening-state-changed",
        (event) => {
          if (!isMounted) return;
          const payload = event.payload;
          if (payload.state === "listening") {
            setState("active-listening");
            setIsVisible(true);
            // Initialize session state
            if (payload.session_id) {
              setActiveListeningSessionId(payload.session_id);
              setSessionStartTime(Date.now());
              setActiveListeningInsights([]);
              setCurrentTranscription("");
            }
          } else if (payload.state === "processing") {
            setState("active-listening-processing");
          } else if (payload.state === "idle") {
            setIsVisible(false);
            // Clear session state
            setActiveListeningSessionId(null);
            setSessionStartTime(null);
            setActiveListeningInsights([]);
            setCurrentTranscription("");
          } else if (payload.state === "error" && payload.error) {
            console.error("Active listening error:", payload.error);
          }
        }
      );
      unlistenFns.push(unlistenALState);

      // Listen for active listening segment transcriptions
      const unlistenALSegment = await listen<ActiveListeningSegmentEvent>(
        "active-listening-segment",
        (event) => {
          if (!isMounted) return;
          const payload = event.payload;
          setCurrentTranscription(payload.transcription);
          // Add a new insight placeholder that will be filled when insight arrives
          const insightId = `${payload.session_id}-${payload.timestamp}`;
          setActiveListeningInsights((prev) => {
            // Check if insight already exists
            if (prev.some((i) => i.id === insightId)) {
              return prev;
            }
            return [
              ...prev,
              {
                id: insightId,
                transcription: payload.transcription,
                insight: "",
                timestamp: payload.timestamp,
                isStreaming: true,
                speakerId: payload.speaker_id,
                speakerLabel: payload.speaker_label,
              },
            ];
          });
        }
      );
      unlistenFns.push(unlistenALSegment);

      // Listen for active listening insight chunks
      const unlistenALInsight = await listen<ActiveListeningInsightEvent>(
        "active-listening-insight",
        (event) => {
          if (!isMounted) return;
          const payload = event.payload;
          setActiveListeningInsights((prev) => {
            // Find the most recent insight that is still streaming
            const lastIndex = prev.findIndex((i) => i.isStreaming);
            if (lastIndex === -1) return prev;

            const updated = [...prev];
            if (payload.done) {
              updated[lastIndex] = {
                ...updated[lastIndex],
                isStreaming: false,
              };
            } else {
              updated[lastIndex] = {
                ...updated[lastIndex],
                insight: updated[lastIndex].insight + payload.chunk,
              };
            }
            return updated;
          });
        }
      );
      unlistenFns.push(unlistenALInsight);

      // Listen for Ask AI state changes
      const unlistenAskAiState = await listen<AskAiStateEvent>(
        "ask-ai-state-changed",
        (event) => {
          if (!isMounted) return;
          const payload = event.payload;

          if (payload.question) {
            setAskAiQuestion(payload.question);
          }

          if (payload.error) {
            setAskAiError(payload.error);
          }

          // Update conversation state
          if (payload.conversation) {
            setConversation(payload.conversation);
          }

          // Map Ask AI state to overlay state
          switch (payload.state) {
            case "recording":
              setState("ask-ai-recording");
              setIsVisible(true);
              break;
            case "transcribing":
              setState("ask-ai-transcribing");
              break;
            case "generating":
              setState("ask-ai-generating");
              break;
            case "complete":
              setState("ask-ai-complete");
              break;
            case "conversation_active":
              setState("ask-ai-complete");
              break;
            case "error":
              setState("ask-ai-error");
              break;
            case "idle":
              // Reset conversation on idle
              setConversation(null);
              break;
          }
        }
      );
      unlistenFns.push(unlistenAskAiState);

      // Listen for Ask AI response chunks
      const unlistenAskAiResponse = await listen<AskAiResponseEvent>(
        "ask-ai-response",
        (event) => {
          if (!isMounted) return;
          const payload = event.payload;

          if (!payload.done && payload.chunk) {
            setAskAiResponse((prev) => prev + payload.chunk);
          }
        }
      );
      unlistenFns.push(unlistenAskAiResponse);
    };

    setupEventListeners();

    // Cleanup function
    return () => {
      isMounted = false;
      unlistenFns.forEach((unlisten) => {
        if (unlisten) unlisten();
      });
    };
  }, []);

  const getIcon = () => {
    if (
      state === "recording" ||
      state === "active-listening" ||
      state === "ask-ai-recording"
    ) {
      return <MicrophoneIcon />;
    } else {
      return <TranscriptionIcon />;
    }
  };

  const isActiveListening =
    state === "active-listening" || state === "active-listening-processing";
  const isAskAiRecording =
    state === "ask-ai-recording" || state === "ask-ai-transcribing";
  const isAskAiResponse =
    state === "ask-ai-generating" ||
    state === "ask-ai-complete" ||
    state === "ask-ai-error";
  const isRecording =
    state === "recording" ||
    state === "active-listening" ||
    state === "ask-ai-recording";
  const isProcessing =
    state === "transcribing" ||
    state === "active-listening-processing" ||
    state === "ask-ai-transcribing";

  // Generate accessible status text for screen readers
  const getStatusText = () => {
    if (isActiveListening && isProcessing) {
      return t(
        "overlay.status.activeListeningProcessing",
        "Active listening, processing audio"
      );
    }
    if (isActiveListening) {
      return t(
        "overlay.status.activeListening",
        "Active listening mode enabled"
      );
    }
    if (isAskAiResponse) {
      return t("overlay.status.askAiResponse", "Ask AI response");
    }
    if (isAskAiRecording && isProcessing) {
      return t("overlay.status.askAiProcessing", "Ask AI, processing question");
    }
    if (isAskAiRecording) {
      return t("overlay.status.askAiRecording", "Ask AI, recording question");
    }
    if (isProcessing) {
      return t("overlay.status.transcribing", "Transcribing audio");
    }
    return t("overlay.status.recording", "Recording audio");
  };

  const handleDismiss = async () => {
    if (autoDismissRef.current) {
      clearTimeout(autoDismissRef.current);
    }
    await commands.dismissAskAiSession();
    setIsVisible(false);
    // Reset state after fade out
    setTimeout(() => {
      setAskAiQuestion("");
      setAskAiResponse("");
      setAskAiError(null);
      setCopied(false);
      setConversation(null);
    }, 300);
  };

  const handleNewConversation = async () => {
    if (autoDismissRef.current) {
      clearTimeout(autoDismissRef.current);
    }
    await commands.startNewAskAiConversation();
    // Clear current state but keep overlay visible
    setAskAiQuestion("");
    setAskAiResponse("");
    setAskAiError(null);
    setCopied(false);
    setConversation(null);
  };

  const handleCopy = async () => {
    if (askAiResponse) {
      await writeText(askAiResponse);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  // Error type detection for showing appropriate actions
  const getErrorType = (error: string): "connection" | "config" | "transient" | "speech" | "unknown" => {
    if (error.includes("connection refused") || error.includes("Failed to connect")) {
      return "connection";
    }
    if (error.includes("No Ollama model configured") || error.includes("No model")) {
      return "config";
    }
    if (error.includes("timeout") || error.includes("timed out") || error.includes("model may be loading")) {
      return "transient";
    }
    if (error.includes("Transcription failed") || error.includes("No speech detected")) {
      return "speech";
    }
    return "unknown";
  };

  const formatErrorMessage = (error: string): string => {
    if (
      error.includes("connection refused") ||
      error.includes("Failed to connect")
    ) {
      return t(
        "askAi.errors.connectionFailed",
        "Could not connect to Ollama. Please ensure Ollama is running."
      );
    }
    if (error.includes("No Ollama model configured") || error.includes("No model")) {
      return t(
        "askAi.errors.noModel",
        "No AI model selected. Please configure a model in Ask AI settings."
      );
    }
    if (error.includes("Transcription failed")) {
      return t(
        "askAi.errors.transcriptionFailed",
        "Failed to transcribe audio. Please try speaking more clearly."
      );
    }
    if (error.includes("No speech detected")) {
      return t(
        "askAi.errors.noSpeech",
        "No speech was detected. Please speak clearly and try again."
      );
    }
    if (error.includes("timeout") || error.includes("timed out")) {
      return t(
        "askAi.errors.timeout",
        "The request timed out. The AI model may be loading or busy."
      );
    }
    return error.length > 100 ? error.substring(0, 100) + "..." : error;
  };

  const handleRetry = async () => {
    // Reset error state and clear current question for retry
    setAskAiError(null);
    await commands.dismissAskAiSession();
    // User can trigger shortcut again manually
  };

  // Format session duration
  const formatDuration = (startTime: number | null): string => {
    if (!startTime) return "0:00";
    const seconds = Math.floor((Date.now() - startTime) / 1000);
    const minutes = Math.floor(seconds / 60);
    const remainingSeconds = seconds % 60;
    return `${minutes}:${remainingSeconds.toString().padStart(2, "0")}`;
  };

  // Render expanded Active Listening insights view
  if (isActiveListening && activeListeningInsights.length > 0) {
    return (
      <div
        className={`recording-overlay active-listening-expanded ${isVisible ? "fade-in" : ""}`}
        role="dialog"
        aria-live="polite"
        aria-label={getStatusText()}
      >
        <div className="active-listening-container">
          {/* Header */}
          <div className="active-listening-header">
            <span className="active-listening-header-title">
              {t("activeListening.overlay.title", "Active Listening")}
            </span>
            <div className="active-listening-header-stats">
              <span className="active-listening-stat">
                <span className="pulse-dot" aria-hidden="true" />
                {formatDuration(sessionStartTime)}
              </span>
              <span className="active-listening-stat">
                {t("activeListening.overlay.insights", "{{count}} insights", {
                  count: activeListeningInsights.filter((i) => i.insight).length,
                })}
              </span>
            </div>
          </div>

          {/* Current transcription */}
          {currentTranscription && isProcessing && (
            <div className="active-listening-current">
              <span className="active-listening-current-label">
                {t("activeListening.overlay.processing", "Processing")}
              </span>
              <span className="active-listening-current-text">
                {currentTranscription}
              </span>
            </div>
          )}

          {/* Insights list */}
          <div className="active-listening-insights-scroll" ref={insightsScrollRef}>
            {activeListeningInsights.map((item) => (
              <div
                key={item.id}
                className={`active-listening-insight-item ${item.isStreaming ? "streaming" : ""} ${item.speakerId !== null ? `speaker-${item.speakerId}` : ""}`}
              >
                <div className="active-listening-insight-transcription">
                  {item.speakerLabel && (
                    <span className={`active-listening-speaker-label speaker-${item.speakerId ?? 0}`}>
                      [{item.speakerLabel}]
                    </span>
                  )}
                  <span className="active-listening-insight-quote">"</span>
                  {item.transcription}
                  <span className="active-listening-insight-quote">"</span>
                </div>
                <div className="active-listening-insight-content">
                  {item.insight || (
                    <span className="active-listening-insight-waiting">
                      {t("activeListening.overlay.generating", "Generating insight...")}
                    </span>
                  )}
                  {item.isStreaming && item.insight && (
                    <span className="typing-cursor" aria-hidden="true" />
                  )}
                </div>
              </div>
            ))}
          </div>

          {/* Status bar */}
          <div className="active-listening-status">
            {isProcessing ? (
              <span className="active-listening-status-text processing">
                {t("activeListening.overlay.processingSegment", "Processing segment...")}
              </span>
            ) : (
              <span className="active-listening-status-text listening">
                {t("activeListening.overlay.listening", "Listening for speech...")}
              </span>
            )}
          </div>
        </div>
      </div>
    );
  }

  // Render expanded Ask AI response view
  if (isAskAiResponse) {
    const hasPreviousTurns = conversation && conversation.turns.length > 0;

    return (
      <div
        className={`recording-overlay ask-ai-expanded ${isVisible ? "fade-in" : ""}`}
        role="dialog"
        aria-live="polite"
        aria-label={getStatusText()}
      >
        <div className="ask-ai-response-container">
          {/* Draggable header */}
          <div
            className="ask-ai-header"
            onMouseDown={handleDragStart}
          >
            <span className="ask-ai-header-title">{t("askAi.title", "Ask AI")}</span>
            <div className="ask-ai-header-actions">
              {hasPreviousTurns && (
                <button
                  type="button"
                  className="ask-ai-new-button"
                  onClick={handleNewConversation}
                  aria-label={t("askAi.newConversation", "New conversation")}
                  title={t("askAi.newConversation", "New conversation")}
                >
                  <svg
                    width="14"
                    height="14"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="2"
                  >
                    <line x1="12" y1="5" x2="12" y2="19" />
                    <line x1="5" y1="12" x2="19" y2="12" />
                  </svg>
                </button>
              )}
              <button
                type="button"
                className="ask-ai-close-button"
                onClick={handleDismiss}
                aria-label={t("common.close", "Close")}
              >
                <CancelIcon />
              </button>
            </div>
          </div>

          {/* Scrollable conversation */}
          <div className="ask-ai-conversation-scroll" ref={responseRef}>
            {/* Previous turns from conversation history */}
            {hasPreviousTurns && conversation.turns.map((turn: ConversationTurn) => (
              <div key={turn.id} className="ask-ai-turn">
                <div className="ask-ai-q">
                  <span className="ask-ai-label">{t("askAi.question", "Q:")}</span>
                  <span className="ask-ai-question-text">{turn.question}</span>
                </div>
                <div className="ask-ai-a">{turn.response}</div>
              </div>
            ))}

            {/* Current turn (question being asked or response generating) */}
            {askAiQuestion && (
              <div className="ask-ai-turn current">
                <div className="ask-ai-q">
                  <span className="ask-ai-label">{t("askAi.question", "Q:")}</span>
                  <span className="ask-ai-question-text">{askAiQuestion}</span>
                </div>
                {state === "ask-ai-error" && askAiError ? (
                  <div className="ask-ai-error-inline">
                    <div className="ask-ai-error-content">
                      <div className="ask-ai-error-message">
                        <div className="ask-ai-error-icon">
                          <svg
                            width="16"
                            height="16"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            strokeWidth="2"
                          >
                            <circle cx="12" cy="12" r="10" />
                            <line x1="12" y1="8" x2="12" y2="12" />
                            <line x1="12" y1="16" x2="12.01" y2="16" />
                          </svg>
                        </div>
                        <span className="ask-ai-error-text">
                          {formatErrorMessage(askAiError)}
                        </span>
                      </div>
                      {/* Show troubleshooting hint for connection/config errors */}
                      {(getErrorType(askAiError) === "connection" || getErrorType(askAiError) === "config") && (
                        <div className="ask-ai-error-hint">
                          {t("askAi.errors.checkSettings", "Check Ask AI settings to configure Ollama.")}
                        </div>
                      )}
                      {/* Show retry button for transient errors */}
                      {(getErrorType(askAiError) === "transient" || getErrorType(askAiError) === "speech") && (
                        <button
                          type="button"
                          className="ask-ai-retry-button"
                          onClick={handleRetry}
                        >
                          {t("askAi.errors.retry", "Try Again")}
                        </button>
                      )}
                    </div>
                  </div>
                ) : (
                  <div className="ask-ai-a">
                    {askAiResponse || (
                      <span className="ask-ai-waiting">
                        {t("askAi.waitingResponse", "Waiting for response...")}
                      </span>
                    )}
                    {state === "ask-ai-generating" && (
                      <span className="typing-cursor" aria-hidden="true" />
                    )}
                  </div>
                )}
              </div>
            )}

            {/* Show error when no question (e.g., empty transcription) */}
            {!askAiQuestion && state === "ask-ai-error" && askAiError && (
              <div className="ask-ai-error">
                <div className="ask-ai-error-content">
                  <div className="ask-ai-error-message">
                    <div className="ask-ai-error-icon">
                      <svg
                        width="20"
                        height="20"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        strokeWidth="2"
                      >
                        <circle cx="12" cy="12" r="10" />
                        <line x1="12" y1="8" x2="12" y2="12" />
                        <line x1="12" y1="16" x2="12.01" y2="16" />
                      </svg>
                    </div>
                    <span className="ask-ai-error-text">
                      {formatErrorMessage(askAiError)}
                    </span>
                  </div>
                  {/* Show troubleshooting hint for connection/config errors */}
                  {(getErrorType(askAiError) === "connection" || getErrorType(askAiError) === "config") && (
                    <div className="ask-ai-error-hint">
                      {t("askAi.errors.checkSettings", "Check Ask AI settings to configure Ollama.")}
                    </div>
                  )}
                  {/* Show retry button for transient errors */}
                  {(getErrorType(askAiError) === "transient" || getErrorType(askAiError) === "speech") && (
                    <button
                      type="button"
                      className="ask-ai-retry-button"
                      onClick={handleRetry}
                    >
                      {t("askAi.errors.retry", "Try Again")}
                    </button>
                  )}
                </div>
              </div>
            )}
          </div>

          {/* Hint when waiting for follow-up */}
          {state === "ask-ai-complete" && (
            <div className="ask-ai-hint">
              {t("askAi.followUpHint", "Press shortcut to continue conversation...")}
            </div>
          )}

          {/* Actions bar */}
          <div className="ask-ai-actions">
            {askAiResponse && (
              <button
                type="button"
                className="ask-ai-copy-button"
                onClick={handleCopy}
                aria-label={t("askAi.copy", "Copy response")}
                title={t("askAi.copy", "Copy response")}
              >
                {copied ? (
                  <svg
                    width="16"
                    height="16"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="2"
                  >
                    <polyline points="20 6 9 17 4 12" />
                  </svg>
                ) : (
                  <svg
                    width="16"
                    height="16"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="2"
                  >
                    <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
                    <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
                  </svg>
                )}
              </button>
            )}
            {/* Status indicator */}
            <div className="ask-ai-status-inline">
              {state === "ask-ai-generating" && (
                <span className="ask-ai-status-dot generating" />
              )}
              {state === "ask-ai-complete" && (
                <span className="ask-ai-status-dot complete" />
              )}
              {state === "ask-ai-error" && (
                <span className="ask-ai-status-dot error" />
              )}
            </div>
          </div>
        </div>
      </div>
    );
  }

  // Render compact recording overlay
  return (
    <div
      className={`recording-overlay ${isVisible ? "fade-in" : ""}`}
      role="status"
      aria-live="polite"
      aria-label={getStatusText()}
    >
      <div className="overlay-left" aria-hidden="true">
        {getIcon()}
      </div>

      <div className="overlay-middle">
        {isRecording && (
          <div
            className="bars-container"
            role="img"
            aria-label={t("overlay.audioLevels", "Audio level indicator")}
          >
            {levels.map((v, i) => (
              <div
                key={i}
                className="bar"
                style={{
                  height: `${Math.min(20, 4 + Math.pow(v, 0.7) * 16)}px`, // Cap at 20px max height
                  transition: "height 60ms ease-out, opacity 120ms ease-out",
                  opacity: Math.max(0.2, v * 1.7), // Minimum opacity for visibility
                }}
              />
            ))}
          </div>
        )}
        {isProcessing && (
          <div className="transcribing-text" aria-hidden="true">
            {state === "active-listening-processing"
              ? t("overlay.processing", "Processing...")
              : state === "ask-ai-transcribing"
                ? t("overlay.askAiProcessing", "Ask AI...")
                : t("overlay.transcribing")}
          </div>
        )}
      </div>

      <div className="overlay-right">
        {state === "recording" && (
          <button
            type="button"
            className="cancel-button"
            onClick={() => {
              commands.cancelOperation();
            }}
            aria-label={t("overlay.cancel", "Cancel recording")}
          >
            <CancelIcon />
          </button>
        )}
        {isActiveListening && (
          <div
            className="active-listening-indicator"
            role="img"
            aria-label={t(
              "overlay.activeListeningIndicator",
              "Active listening indicator"
            )}
          >
            <span className="pulse-dot" aria-hidden="true" />
          </div>
        )}
        {isAskAiRecording && (
          <div
            className="ask-ai-indicator"
            role="img"
            aria-label={t("overlay.askAiIndicator", "Ask AI indicator")}
          >
            <span className="pulse-dot ask-ai-dot" aria-hidden="true" />
          </div>
        )}
      </div>
    </div>
  );
};

export default RecordingOverlay;
