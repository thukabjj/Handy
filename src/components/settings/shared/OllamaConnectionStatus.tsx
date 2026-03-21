import React, { useEffect, useState, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { RefreshCcw, Wifi, WifiOff } from "lucide-react";

interface OllamaConnectionStatusProps {
  /** Optional callback when connection status changes */
  onStatusChange?: (connected: boolean) => void;
  /** Optional endpoint to check */
  endpoint?: string;
}

/**
 * Shared component for displaying Ollama connection status.
 * Shows connection state with a refresh button.
 *
 * Note: This is a placeholder component. The checkOllamaConnection command
 * needs to be implemented in the backend before this will work.
 */
export const OllamaConnectionStatus: React.FC<OllamaConnectionStatusProps> = ({
  onStatusChange,
  endpoint = "http://localhost:11434",
}) => {
  const { t } = useTranslation();
  const [isConnected, setIsConnected] = useState<boolean | null>(null);
  const [isChecking, setIsChecking] = useState(false);

  const checkConnection = useCallback(async () => {
    setIsChecking(true);
    try {
      // Try a simple fetch to the Ollama API
      const response = await fetch(`${endpoint}/api/tags`, {
        method: "GET",
        signal: AbortSignal.timeout(5000),
      });
      const connected = response.ok;
      setIsConnected(connected);
      onStatusChange?.(connected);
    } catch {
      setIsConnected(false);
      onStatusChange?.(false);
    } finally {
      setIsChecking(false);
    }
  }, [endpoint, onStatusChange]);

  useEffect(() => {
    checkConnection();
  }, [checkConnection]);

  const statusText = isChecking
    ? t("shared.ollama.connectionStatus.checking")
    : isConnected
      ? t("shared.ollama.connectionStatus.connected")
      : t("shared.ollama.connectionStatus.disconnected");

  const StatusIcon = isConnected ? Wifi : WifiOff;
  const statusColor = isConnected ? "text-green-500" : "text-red-500";

  return (
    <div className="flex items-center gap-2 p-3 bg-mid-gray/5 rounded-lg border border-mid-gray/20">
      <StatusIcon className={`h-4 w-4 ${statusColor}`} />
      <span className={`text-sm ${statusColor}`}>{statusText}</span>
      <button
        onClick={checkConnection}
        disabled={isChecking}
        className="ml-auto p-1 hover:bg-mid-gray/20 rounded transition-colors"
        title={t("shared.ollama.refreshConnection")}
        aria-label={t("shared.ollama.refreshConnection")}
      >
        <RefreshCcw className={`h-4 w-4 ${isChecking ? "animate-spin" : ""}`} />
      </button>
    </div>
  );
};
