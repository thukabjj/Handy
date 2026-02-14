import React, { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  FolderInput,
  Play,
  Square,
  Trash2,
  X,
  FileAudio,
  Check,
  AlertCircle,
  Loader2,
} from "lucide-react";
import { SettingsGroup } from "@/components/ui";
import { Button } from "@/components/ui/Button";

interface BatchItem {
  id: string;
  file_name: string;
  file_path: string;
  status:
    | "Queued"
    | "Decoding"
    | "Transcribing"
    | "Completed"
    | "Failed"
    | "Cancelled";
  progress: number;
  error: string | null;
  duration_seconds: number | null;
}

interface BatchItemStatusEvent {
  id: string;
  status: BatchItem["status"];
  progress: number;
  error: string | null;
  duration_seconds: number | null;
}

const AUDIO_EXTENSIONS = ["wav", "mp3", "m4a", "aac", "flac", "ogg", "mp4"];

function formatDuration(seconds: number): string {
  const mins = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  return `${mins}:${secs.toString().padStart(2, "0")}`;
}

const StatusBadge: React.FC<{ status: BatchItem["status"] }> = ({
  status,
}) => {
  const { t } = useTranslation();

  const styles: Record<BatchItem["status"], string> = {
    Queued: "bg-mid-gray/20 text-text-secondary",
    Decoding: "bg-blue-500/20 text-blue-400",
    Transcribing: "bg-primary-light/20 text-primary-light",
    Completed: "bg-green-500/20 text-green-400",
    Failed: "bg-red-500/20 text-red-400",
    Cancelled: "bg-mid-gray/20 text-text-secondary",
  };

  return (
    <span
      className={`inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-xs font-medium ${styles[status]}`}
    >
      {status === "Decoding" || status === "Transcribing" ? (
        <Loader2 className="h-3 w-3 animate-spin" />
      ) : status === "Completed" ? (
        <Check className="h-3 w-3" />
      ) : status === "Failed" ? (
        <AlertCircle className="h-3 w-3" />
      ) : null}
      {t(`batchProcessing.status.${status.toLowerCase()}`)}
    </span>
  );
};

const ProgressBar: React.FC<{ progress: number; status: BatchItem["status"] }> =
  ({ progress, status }) => {
    const barColor =
      status === "Failed"
        ? "bg-red-500"
        : status === "Completed"
          ? "bg-green-500"
          : "bg-primary-light";

    return (
      <div className="h-1.5 w-full rounded-full bg-mid-gray/20">
        <div
          className={`h-full rounded-full transition-all duration-300 ${barColor}`}
          style={{ width: `${Math.min(100, Math.max(0, progress))}%` }}
        />
      </div>
    );
  };

export const BatchProcessingPanel: React.FC = () => {
  const { t } = useTranslation();
  const [items, setItems] = useState<BatchItem[]>([]);
  const [isProcessing, setIsProcessing] = useState(false);

  useEffect(() => {
    const unlistenStatus = listen<BatchItemStatusEvent>(
      "batch-item-status",
      (event) => {
        setItems((prev) =>
          prev.map((item) =>
            item.id === event.payload.id
              ? {
                  ...item,
                  status: event.payload.status,
                  progress: event.payload.progress,
                  error: event.payload.error,
                  duration_seconds: event.payload.duration_seconds,
                }
              : item,
          ),
        );
      },
    );

    const unlistenComplete = listen("batch-complete", () => {
      setIsProcessing(false);
    });

    return () => {
      unlistenStatus.then((fn) => fn());
      unlistenComplete.then((fn) => fn());
    };
  }, []);

  const handleSelectFiles = useCallback(async () => {
    const selected = await open({
      multiple: true,
      filters: [
        {
          name: t("batchProcessing.audioFiles"),
          extensions: AUDIO_EXTENSIONS,
        },
      ],
    });

    if (!selected) return;

    const paths = Array.isArray(selected) ? selected : [selected];
    if (paths.length === 0) return;

    try {
      const newItems = await invoke<BatchItem[]>("add_to_batch_queue", {
        paths,
      });
      setItems((prev) => [...prev, ...newItems]);
    } catch (error) {
      console.error("Failed to add files to batch queue:", error);
    }
  }, [t]);

  const handleStart = useCallback(async () => {
    try {
      setIsProcessing(true);
      await invoke("start_batch_processing");
    } catch (error) {
      setIsProcessing(false);
      console.error("Failed to start batch processing:", error);
    }
  }, []);

  const handleCancel = useCallback(async () => {
    try {
      await invoke("cancel_batch_processing");
      setIsProcessing(false);
    } catch (error) {
      console.error("Failed to cancel batch processing:", error);
    }
  }, []);

  const handleClearCompleted = useCallback(async () => {
    try {
      await invoke("clear_completed_batch_items");
      setItems((prev) =>
        prev.filter(
          (item) => item.status !== "Completed" && item.status !== "Failed" && item.status !== "Cancelled",
        ),
      );
    } catch (error) {
      console.error("Failed to clear completed items:", error);
    }
  }, []);

  const handleRemoveItem = useCallback(async (id: string) => {
    try {
      await invoke("remove_batch_item", { id });
      setItems((prev) => prev.filter((item) => item.id !== id));
    } catch (error) {
      console.error("Failed to remove batch item:", error);
    }
  }, []);

  const completedCount = items.filter(
    (item) => item.status === "Completed",
  ).length;
  const failedCount = items.filter((item) => item.status === "Failed").length;
  const totalCount = items.length;
  const overallProgress =
    totalCount > 0
      ? items.reduce((sum, item) => sum + item.progress, 0) / totalCount
      : 0;
  const hasQueuedItems = items.some(
    (item) =>
      item.status === "Queued" ||
      item.status === "Decoding" ||
      item.status === "Transcribing",
  );
  const hasFinishedItems = items.some(
    (item) =>
      item.status === "Completed" ||
      item.status === "Failed" ||
      item.status === "Cancelled",
  );

  return (
    <div className="max-w-3xl w-full mx-auto space-y-6">
      <SettingsGroup
        title={t("batchProcessing.title")}
        description={t("batchProcessing.description")}
      >
        {/* Action buttons */}
        <div className="flex items-center gap-2 p-4">
          <Button
            onClick={handleSelectFiles}
            variant="secondary"
            size="md"
            disabled={isProcessing}
          >
            <FolderInput className="h-4 w-4" />
            {t("batchProcessing.selectFiles")}
          </Button>

          {!isProcessing ? (
            <Button
              onClick={handleStart}
              variant="primary"
              size="md"
              disabled={!hasQueuedItems}
            >
              <Play className="h-4 w-4" />
              {t("batchProcessing.start")}
            </Button>
          ) : (
            <Button onClick={handleCancel} variant="danger" size="md">
              <Square className="h-4 w-4" />
              {t("batchProcessing.cancel")}
            </Button>
          )}

          {hasFinishedItems && (
            <Button
              onClick={handleClearCompleted}
              variant="ghost"
              size="md"
              disabled={isProcessing}
            >
              <Trash2 className="h-4 w-4" />
              {t("batchProcessing.clearCompleted")}
            </Button>
          )}
        </div>

        {/* Overall progress */}
        {totalCount > 0 && (
          <div className="px-4 pb-3 space-y-2">
            <ProgressBar progress={overallProgress} status="Transcribing" />
            <p className="text-xs text-text-secondary">
              {t("batchProcessing.stats", {
                completed: completedCount,
                total: totalCount,
                failed: failedCount,
              })}
            </p>
          </div>
        )}

        {/* Queue list */}
        {items.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-12 px-4 text-center">
            <FileAudio className="h-10 w-10 text-mid-gray/40 mb-3" />
            <p className="text-sm text-text-secondary">
              {t("batchProcessing.empty")}
            </p>
          </div>
        ) : (
          <div className="divide-y divide-mid-gray/20">
            {items.map((item) => (
              <div
                key={item.id}
                className="flex items-center gap-3 px-4 py-3 group"
              >
                <FileAudio className="h-4 w-4 flex-shrink-0 text-text-secondary" />

                <div className="flex-1 min-w-0 space-y-1">
                  <div className="flex items-center gap-2">
                    <span className="text-sm text-text truncate">
                      {item.file_name}
                    </span>
                    <StatusBadge status={item.status} />
                  </div>

                  {(item.status === "Decoding" ||
                    item.status === "Transcribing") && (
                    <ProgressBar
                      progress={item.progress}
                      status={item.status}
                    />
                  )}

                  {item.error && (
                    <p className="text-xs text-red-400 truncate">
                      {item.error}
                    </p>
                  )}
                </div>

                {item.duration_seconds != null && (
                  <span className="text-xs text-text-secondary flex-shrink-0">
                    {formatDuration(item.duration_seconds)}
                  </span>
                )}

                <button
                  onClick={() => handleRemoveItem(item.id)}
                  disabled={
                    item.status === "Decoding" ||
                    item.status === "Transcribing"
                  }
                  className="p-1 rounded opacity-0 group-hover:opacity-100 hover:bg-mid-gray/20 transition-all disabled:opacity-30 disabled:cursor-not-allowed"
                  aria-label={t("batchProcessing.removeItem")}
                >
                  <X className="h-3.5 w-3.5 text-text-secondary" />
                </button>
              </div>
            ))}
          </div>
        )}
      </SettingsGroup>
    </div>
  );
};
