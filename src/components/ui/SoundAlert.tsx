import React, { useEffect, useState, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { listen } from "@tauri-apps/api/event";

interface SoundDetectedPayload {
  category: string;
  confidence: number;
  timestamp_ms: number;
}

interface AlertItem {
  id: number;
  category: string;
  confidence: number;
}

const CATEGORY_ICONS: Record<string, string> = {
  doorbell: "🔔",
  alarm: "🚨",
  phone_ring: "📱",
  dog_bark: "🐕",
  baby_cry: "👶",
  knocking: "🚪",
  siren: "🚑",
  applause: "👏",
};

let alertIdCounter = 0;

export const SoundAlert: React.FC = () => {
  const { t } = useTranslation();
  const [alerts, setAlerts] = useState<AlertItem[]>([]);

  const dismissAlert = useCallback((id: number) => {
    setAlerts((prev) => prev.filter((a) => a.id !== id));
  }, []);

  useEffect(() => {
    const unlisten = listen<SoundDetectedPayload>(
      "sound-detected",
      (event) => {
        const id = ++alertIdCounter;
        const alert: AlertItem = {
          id,
          category: event.payload.category,
          confidence: event.payload.confidence,
        };

        setAlerts((prev) => [...prev, alert]);

        // Auto-dismiss after 3 seconds
        setTimeout(() => dismissAlert(id), 3000);
      },
    );

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [dismissAlert]);

  if (alerts.length === 0) return null;

  return (
    <div className="fixed top-4 right-4 z-50 flex flex-col gap-2">
      {alerts.map((alert) => (
        <div
          key={alert.id}
          className="flex items-center gap-2 rounded-lg bg-neutral-800/95 border border-neutral-700 px-4 py-2 shadow-lg backdrop-blur-sm animate-in slide-in-from-right"
        >
          <span className="text-xl">
            {CATEGORY_ICONS[alert.category] ?? "🔊"}
          </span>
          <div>
            <div className="text-sm font-medium text-neutral-200 capitalize">
              {alert.category.replace("_", " ")}
            </div>
            <div className="text-xs text-neutral-400">
              {t("soundDetection.confidence", { value: Math.round(alert.confidence * 100) })}
            </div>
          </div>
        </div>
      ))}
    </div>
  );
};
