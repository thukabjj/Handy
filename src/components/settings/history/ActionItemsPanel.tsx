import React, { useState, useEffect, useCallback, useRef } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { SettingsGroup } from "@/components/ui";
import { Button } from "@/components/ui/Button";
import {
  CheckSquare,
  Square,
  Trash2,
  Download,
  User,
  Calendar,
  Flag,
} from "lucide-react";

interface ActionItem {
  id: number;
  entry_id: number;
  task: string;
  assignee: string | null;
  deadline: string | null;
  priority: string;
  completed: boolean;
  created_at: string;
}

const PRIORITY_COLORS: Record<string, string> = {
  high: "bg-red-500/20 text-red-400",
  medium: "bg-yellow-500/20 text-yellow-400",
  low: "bg-green-500/20 text-green-400",
};

export const ActionItemsPanel: React.FC = () => {
  const { t } = useTranslation();
  const [items, setItems] = useState<ActionItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [exportOpen, setExportOpen] = useState(false);
  const [copyNotice, setCopyNotice] = useState(false);
  const exportRef = useRef<HTMLDivElement>(null);

  const loadItems = useCallback(async () => {
    try {
      const result = await invoke<ActionItem[]>("get_action_items", {
        entryId: null,
      });
      setItems(result);
    } catch (error) {
      console.error("Failed to load action items:", error);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadItems();
  }, [loadItems]);

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (
        exportRef.current &&
        !exportRef.current.contains(event.target as Node)
      ) {
        setExportOpen(false);
      }
    };
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

  const toggleItem = useCallback(
    async (item: ActionItem) => {
      try {
        await invoke("toggle_action_item", {
          id: item.id,
          completed: !item.completed,
        });
        setItems((prev) =>
          prev.map((i) =>
            i.id === item.id ? { ...i, completed: !i.completed } : i,
          ),
        );
      } catch (error) {
        console.error("Failed to toggle action item:", error);
      }
    },
    [],
  );

  const deleteItem = useCallback(
    async (id: number) => {
      try {
        await invoke("delete_action_item", { id });
        setItems((prev) => prev.filter((i) => i.id !== id));
      } catch (error) {
        console.error("Failed to delete action item:", error);
      }
    },
    [],
  );

  const handleExport = useCallback(
    async (format: "markdown" | "json") => {
      setExportOpen(false);
      try {
        const result = await invoke<string>("export_action_items", {
          entryId: null,
          format,
        });
        await navigator.clipboard.writeText(result);
        setCopyNotice(true);
        setTimeout(() => setCopyNotice(false), 2000);
      } catch (error) {
        console.error("Failed to export action items:", error);
      }
    },
    [],
  );

  const completedCount = items.filter((i) => i.completed).length;

  if (loading) {
    return (
      <SettingsGroup title={t("actionItems.title")}>
        <div className="px-4 py-3 text-center text-text-secondary">
          {t("actionItems.loading")}
        </div>
      </SettingsGroup>
    );
  }

  if (items.length === 0) {
    return (
      <SettingsGroup title={t("actionItems.title")}>
        <div className="px-4 py-3 text-center text-text-secondary">
          {t("actionItems.empty")}
        </div>
      </SettingsGroup>
    );
  }

  return (
    <SettingsGroup
      title={t("actionItems.title")}
      description={t("actionItems.description")}
    >
      <div className="px-4 py-3 flex items-center justify-between border-b border-mid-gray/20">
        <span className="text-xs text-text-secondary">
          {t("actionItems.stats", {
            total: items.length,
            completed: completedCount,
          })}
        </span>
        <div className="flex items-center gap-2">
          {copyNotice && (
            <span className="text-xs text-primary-light">
              {t("actionItems.copiedToClipboard")}
            </span>
          )}
          <div className="relative" ref={exportRef}>
            <Button
              variant="secondary"
              size="sm"
              onClick={() => setExportOpen(!exportOpen)}
            >
              <Download className="w-3.5 h-3.5" />
              {t("actionItems.export")}
            </Button>
            {exportOpen && (
              <div className="absolute right-0 top-full mt-1 bg-background border border-mid-gray/20 rounded-lg shadow-lg z-50 min-w-[150px] py-1">
                <button
                  onClick={() => handleExport("markdown")}
                  className="w-full px-3 py-1.5 text-sm text-left hover:bg-primary-light/10 transition-colors cursor-pointer"
                >
                  {t("actionItems.exportMarkdown")}
                </button>
                <button
                  onClick={() => handleExport("json")}
                  className="w-full px-3 py-1.5 text-sm text-left hover:bg-primary-light/10 transition-colors cursor-pointer"
                >
                  {t("actionItems.exportJson")}
                </button>
              </div>
            )}
          </div>
        </div>
      </div>

      <div className="divide-y divide-mid-gray/20">
        {items.map((item) => (
          <div
            key={item.id}
            className="px-4 py-2.5 flex items-start gap-3 group"
          >
            <button
              onClick={() => toggleItem(item)}
              className="mt-0.5 text-text-secondary hover:text-primary-light transition-colors cursor-pointer flex-shrink-0"
              title={t("actionItems.toggle")}
            >
              {item.completed ? (
                <CheckSquare className="w-4 h-4 text-primary-light" />
              ) : (
                <Square className="w-4 h-4" />
              )}
            </button>

            <div className="flex-1 min-w-0">
              <span
                className={`text-sm ${
                  item.completed
                    ? "line-through text-text-secondary"
                    : "text-text"
                }`}
              >
                {item.task}
              </span>

              <div className="flex flex-wrap items-center gap-1.5 mt-1">
                <span
                  className={`inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-[10px] font-medium ${
                    PRIORITY_COLORS[item.priority] ?? PRIORITY_COLORS.low
                  }`}
                >
                  <Flag className="w-2.5 h-2.5" />
                  {t(`actionItems.priority.${item.priority}`)}
                </span>

                {item.assignee && (
                  <span className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-[10px] font-medium bg-mid-gray/20 text-text-secondary">
                    <User className="w-2.5 h-2.5" />
                    {item.assignee}
                  </span>
                )}

                {item.deadline && (
                  <span className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-[10px] font-medium bg-mid-gray/20 text-text-secondary">
                    <Calendar className="w-2.5 h-2.5" />
                    {item.deadline}
                  </span>
                )}
              </div>
            </div>

            <button
              onClick={() => deleteItem(item.id)}
              className="mt-0.5 text-text-secondary hover:text-red-400 transition-colors cursor-pointer opacity-0 group-hover:opacity-100 flex-shrink-0"
              title={t("actionItems.delete")}
            >
              <Trash2 className="w-3.5 h-3.5" />
            </button>
          </div>
        ))}
      </div>
    </SettingsGroup>
  );
};
