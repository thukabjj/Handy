import React, { useMemo, useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { BarChart3, Users, Clock, MessageSquare, RefreshCcw } from "lucide-react";
import { commands, ActiveListeningSession, SessionInsight } from "@/bindings";
import { SettingsGroup } from "@/components/ui";
import { Button } from "../../ui/Button";

interface SpeakerStats {
  speakerId: number | null;
  label: string;
  totalDurationMs: number;
  turnCount: number;
  wordCount: number;
  percentage: number;
}

interface AnalyticsData {
  speakers: SpeakerStats[];
  totalDurationMs: number;
  totalTurns: number;
  totalWords: number;
  sessionDurationMs: number;
}

const SPEAKER_COLORS = [
  "bg-blue-500",
  "bg-green-500",
  "bg-yellow-500",
  "bg-red-500",
  "bg-purple-500",
  "bg-pink-500",
];

const SPEAKER_TEXT_COLORS = [
  "text-blue-500",
  "text-green-500",
  "text-yellow-500",
  "text-red-500",
  "text-purple-500",
  "text-pink-500",
];

function computeAnalytics(session: ActiveListeningSession): AnalyticsData {
  const speakerMap = new Map<
    string,
    { id: number | null; label: string; durationMs: number; turns: number; words: number }
  >();

  for (const insight of session.insights) {
    const key = insight.speaker_id !== null ? String(insight.speaker_id) : "unknown";
    const label = insight.speaker_label || "Unknown";

    const existing = speakerMap.get(key);
    const wordCount = insight.transcription.trim().split(/\s+/).filter(Boolean).length;

    if (existing) {
      existing.durationMs += insight.duration_ms;
      existing.turns += 1;
      existing.words += wordCount;
    } else {
      speakerMap.set(key, {
        id: insight.speaker_id,
        label,
        durationMs: insight.duration_ms,
        turns: 1,
        words: wordCount,
      });
    }
  }

  const totalDurationMs = Array.from(speakerMap.values()).reduce(
    (sum, s) => sum + s.durationMs,
    0
  );
  const totalTurns = Array.from(speakerMap.values()).reduce(
    (sum, s) => sum + s.turns,
    0
  );
  const totalWords = Array.from(speakerMap.values()).reduce(
    (sum, s) => sum + s.words,
    0
  );

  const sessionDurationMs = session.ended_at
    ? session.ended_at - session.started_at
    : Date.now() - session.started_at;

  const speakers: SpeakerStats[] = Array.from(speakerMap.values())
    .map((s) => ({
      speakerId: s.id,
      label: s.label,
      totalDurationMs: s.durationMs,
      turnCount: s.turns,
      wordCount: s.words,
      percentage: totalDurationMs > 0 ? (s.durationMs / totalDurationMs) * 100 : 0,
    }))
    .sort((a, b) => b.totalDurationMs - a.totalDurationMs);

  return { speakers, totalDurationMs, totalTurns, totalWords, sessionDurationMs };
}

function formatDuration(ms: number): string {
  const totalSeconds = Math.floor(ms / 1000);
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;
  if (minutes > 0) {
    return `${minutes}m ${seconds}s`;
  }
  return `${seconds}s`;
}

const SpeakerBar: React.FC<{ speaker: SpeakerStats; index: number }> = ({
  speaker,
  index,
}) => {
  const { t } = useTranslation();
  const colorClass = SPEAKER_COLORS[index % SPEAKER_COLORS.length];
  const textColorClass = SPEAKER_TEXT_COLORS[index % SPEAKER_TEXT_COLORS.length];

  return (
    <div className="space-y-1.5">
      <div className="flex items-center justify-between text-sm">
        <div className="flex items-center gap-2">
          <div className={`w-3 h-3 rounded-full ${colorClass}`} />
          <span className={`font-medium ${textColorClass}`}>{speaker.label}</span>
        </div>
        <span className="text-mid-gray">
          {speaker.percentage.toFixed(1)}%
        </span>
      </div>
      <div className="w-full bg-mid-gray/10 rounded-full h-2.5">
        <div
          className={`h-2.5 rounded-full ${colorClass} transition-all duration-500`}
          style={{ width: `${Math.max(speaker.percentage, 1)}%` }}
        />
      </div>
      <div className="flex gap-4 text-xs text-mid-gray">
        <span>
          {t("speakerAnalytics.talkTime", "Talk time")}: {formatDuration(speaker.totalDurationMs)}
        </span>
        <span>
          {t("speakerAnalytics.turns", "Turns")}: {speaker.turnCount}
        </span>
        <span>
          {t("speakerAnalytics.words", "Words")}: {speaker.wordCount}
        </span>
      </div>
    </div>
  );
};

const StatCard: React.FC<{
  icon: React.ReactNode;
  label: string;
  value: string;
}> = ({ icon, label, value }) => (
  <div className="flex items-center gap-3 p-3 bg-mid-gray/5 rounded-lg border border-mid-gray/10">
    <div className="text-mid-gray">{icon}</div>
    <div>
      <p className="text-xs text-mid-gray">{label}</p>
      <p className="text-sm font-semibold text-text">{value}</p>
    </div>
  </div>
);

export const SpeakerAnalytics: React.FC = () => {
  const { t } = useTranslation();
  const [session, setSession] = useState<ActiveListeningSession | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  const loadSession = async () => {
    setIsLoading(true);
    try {
      const result = await commands.getActiveListeningSession();
      setSession(result);
    } catch (err) {
      console.error("Failed to load session:", err);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    loadSession();
    // Poll for updates while session is active
    const interval = setInterval(loadSession, 5000);
    return () => clearInterval(interval);
  }, []);

  const analytics = useMemo(() => {
    if (!session || session.insights.length === 0) return null;
    return computeAnalytics(session);
  }, [session]);

  if (isLoading) {
    return (
      <SettingsGroup title={t("speakerAnalytics.title", "Speaker Analytics")}>
        <div className="p-4 text-center text-mid-gray">
          {t("speakerAnalytics.loading", "Loading analytics...")}
        </div>
      </SettingsGroup>
    );
  }

  if (!session || !analytics || analytics.speakers.length === 0) {
    return (
      <SettingsGroup title={t("speakerAnalytics.title", "Speaker Analytics")}>
        <div className="p-6 text-center">
          <BarChart3 className="h-12 w-12 mx-auto text-mid-gray/50 mb-3" />
          <p className="text-mid-gray">
            {t(
              "speakerAnalytics.empty",
              "No speaker data available. Start an Active Listening session to see speaker analytics."
            )}
          </p>
        </div>
      </SettingsGroup>
    );
  }

  return (
    <SettingsGroup title={t("speakerAnalytics.title", "Speaker Analytics")}>
      <div className="space-y-4">
        <div className="flex items-center justify-between">
          <p className="text-sm text-mid-gray">
            {t(
              "speakerAnalytics.description",
              "Talk time, participation, and turn-taking analysis for the current session."
            )}
          </p>
          <Button
            variant="ghost"
            size="sm"
            onClick={loadSession}
            className="text-mid-gray hover:text-text"
          >
            <RefreshCcw className="h-4 w-4" />
          </Button>
        </div>

        {/* Summary stats */}
        <div className="grid grid-cols-2 gap-2 sm:grid-cols-4">
          <StatCard
            icon={<Users className="h-4 w-4" />}
            label={t("speakerAnalytics.speakers", "Speakers")}
            value={String(analytics.speakers.length)}
          />
          <StatCard
            icon={<Clock className="h-4 w-4" />}
            label={t("speakerAnalytics.duration", "Duration")}
            value={formatDuration(analytics.sessionDurationMs)}
          />
          <StatCard
            icon={<MessageSquare className="h-4 w-4" />}
            label={t("speakerAnalytics.totalTurns", "Total Turns")}
            value={String(analytics.totalTurns)}
          />
          <StatCard
            icon={<BarChart3 className="h-4 w-4" />}
            label={t("speakerAnalytics.totalWords", "Total Words")}
            value={String(analytics.totalWords)}
          />
        </div>

        {/* Speaker breakdown */}
        <div className="space-y-4">
          <h4 className="text-sm font-semibold text-text">
            {t("speakerAnalytics.breakdown", "Speaker Breakdown")}
          </h4>
          {analytics.speakers.map((speaker, index) => (
            <SpeakerBar
              key={speaker.speakerId ?? "unknown"}
              speaker={speaker}
              index={index}
            />
          ))}
        </div>

        {/* Participation balance indicator */}
        {analytics.speakers.length >= 2 && (
          <div className="p-3 bg-mid-gray/5 rounded-lg border border-mid-gray/10">
            <p className="text-xs font-semibold text-mid-gray uppercase tracking-wide mb-2">
              {t("speakerAnalytics.participationBalance", "Participation Balance")}
            </p>
            <div className="flex h-4 rounded-full overflow-hidden">
              {analytics.speakers.map((speaker, index) => (
                <div
                  key={speaker.speakerId ?? "unknown"}
                  className={`${SPEAKER_COLORS[index % SPEAKER_COLORS.length]} transition-all duration-500`}
                  style={{ width: `${speaker.percentage}%` }}
                  title={`${speaker.label}: ${speaker.percentage.toFixed(1)}%`}
                />
              ))}
            </div>
            <div className="flex justify-between mt-1">
              {analytics.speakers.map((speaker, index) => (
                <span
                  key={speaker.speakerId ?? `unknown-${index}`}
                  className={`text-xs ${SPEAKER_TEXT_COLORS[index % SPEAKER_TEXT_COLORS.length]}`}
                >
                  {speaker.label}
                </span>
              ))}
            </div>
          </div>
        )}
      </div>
    </SettingsGroup>
  );
};
