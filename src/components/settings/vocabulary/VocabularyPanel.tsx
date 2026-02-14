import React, { useState, useEffect, useCallback, useMemo } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import { readTextFile, writeTextFile } from "@tauri-apps/plugin-fs";
import { Plus, Trash2, Upload, Download, Search, BookA, Tag } from "lucide-react";

import { SettingsGroup } from "@/components/ui";
import { Button } from "@/components/ui/Button";
import { Input } from "@/components/ui/Input";

interface VocabularyEntry {
  id: number;
  term: string;
  frequency: number;
  source: string;
  category: string | null;
  created_at: string;
}

const CATEGORIES = ["Medical", "Legal", "Tech", "Names", "Custom"] as const;
type Category = (typeof CATEGORIES)[number];

const CATEGORY_COLORS: Record<Category, string> = {
  Medical: "bg-blue-500/20 text-blue-400 border-blue-500/30",
  Legal: "bg-purple-500/20 text-purple-400 border-purple-500/30",
  Tech: "bg-green-500/20 text-green-400 border-green-500/30",
  Names: "bg-orange-500/20 text-orange-400 border-orange-500/30",
  Custom: "bg-mid-gray/20 text-mid-gray border-mid-gray/30",
};

const CategoryBadge: React.FC<{ category: string }> = ({ category }) => {
  const colors =
    CATEGORY_COLORS[category as Category] ?? CATEGORY_COLORS.Custom;
  return (
    <span
      className={`inline-flex items-center gap-1 text-xs px-1.5 py-0.5 rounded border ${colors}`}
    >
      <Tag className="h-3 w-3" />
      {category}
    </span>
  );
};

const SourceBadge: React.FC<{ source: string }> = ({ source }) => (
  <span className="text-xs px-1.5 py-0.5 rounded bg-mid-gray/10 text-text-secondary border border-mid-gray/20">
    {source}
  </span>
);

export const VocabularyPanel: React.FC = () => {
  const { t } = useTranslation();
  const [entries, setEntries] = useState<VocabularyEntry[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [searchQuery, setSearchQuery] = useState("");
  const [newTerm, setNewTerm] = useState("");
  const [newCategory, setNewCategory] = useState<string>("");

  const loadVocabulary = useCallback(async () => {
    setIsLoading(true);
    try {
      const result = await invoke<VocabularyEntry[]>("get_vocabulary");
      setEntries(result);
    } catch (error) {
      console.error("Failed to load vocabulary:", error);
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    loadVocabulary();
  }, [loadVocabulary]);

  const filteredEntries = useMemo(() => {
    if (!searchQuery.trim()) return entries;
    const query = searchQuery.toLowerCase();
    return entries.filter((entry) =>
      entry.term.toLowerCase().includes(query),
    );
  }, [entries, searchQuery]);

  const handleAddTerm = useCallback(async () => {
    const trimmed = newTerm.trim();
    if (!trimmed) return;
    try {
      await invoke("add_vocabulary_term", {
        term: trimmed,
        category: newCategory || null,
      });
      setNewTerm("");
      setNewCategory("");
      await loadVocabulary();
    } catch (error) {
      console.error("Failed to add vocabulary term:", error);
    }
  }, [newTerm, newCategory, loadVocabulary]);

  const handleRemoveTerm = useCallback(
    async (id: number) => {
      try {
        await invoke("remove_vocabulary_term", { id });
        await loadVocabulary();
      } catch (error) {
        console.error("Failed to remove vocabulary term:", error);
      }
    },
    [loadVocabulary],
  );

  const handleImport = useCallback(async () => {
    try {
      const filePath = await open({
        filters: [{ name: "JSON", extensions: ["json"] }],
        multiple: false,
      });
      if (!filePath) return;
      const json = await readTextFile(filePath);
      await invoke("import_vocabulary", { json });
      await loadVocabulary();
    } catch (error) {
      console.error("Failed to import vocabulary:", error);
    }
  }, [loadVocabulary]);

  const handleExport = useCallback(async () => {
    try {
      const json = await invoke<string>("export_vocabulary");
      const filePath = await save({
        filters: [{ name: "JSON", extensions: ["json"] }],
        defaultPath: "vocabulary.json",
      });
      if (!filePath) return;
      await writeTextFile(filePath, json);
    } catch (error) {
      console.error("Failed to export vocabulary:", error);
    }
  }, []);

  return (
    <div className="max-w-3xl w-full mx-auto space-y-6">
      {/* Header and actions */}
      <SettingsGroup
        title={t("vocabulary.title")}
        description={t("vocabulary.description")}
      >
        <div className="p-4 space-y-4">
          {/* Search */}
          <div className="relative">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-mid-gray" />
            <Input
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder={t("vocabulary.searchPlaceholder")}
              className="w-full pl-9"
            />
          </div>

          {/* Add term form */}
          <div className="flex items-center gap-2">
            <Input
              value={newTerm}
              onChange={(e) => setNewTerm(e.target.value)}
              placeholder={t("vocabulary.addPlaceholder")}
              className="flex-1"
              onKeyDown={(e) => e.key === "Enter" && handleAddTerm()}
            />
            <select
              value={newCategory}
              onChange={(e) => setNewCategory(e.target.value)}
              className="px-2 py-1.5 text-sm bg-mid-gray/10 border border-mid-gray/80 rounded-md text-text transition-all duration-150 hover:bg-primary-light/10 hover:border-primary-light focus:outline-none focus:bg-primary-light/20 focus:border-primary-light"
            >
              <option value="">{t("vocabulary.noCategory")}</option>
              {CATEGORIES.map((cat) => (
                <option key={cat} value={cat}>
                  {t(`vocabulary.categories.${cat.toLowerCase()}`)}
                </option>
              ))}
            </select>
            <Button
              onClick={handleAddTerm}
              variant="primary"
              size="md"
              disabled={!newTerm.trim()}
              aria-label={t("vocabulary.add")}
            >
              <Plus className="h-4 w-4" />
              {t("vocabulary.add")}
            </Button>
          </div>

          {/* Import / Export */}
          <div className="flex items-center gap-2">
            <Button onClick={handleImport} variant="secondary" size="sm">
              <Upload className="h-4 w-4" />
              {t("vocabulary.import")}
            </Button>
            <Button
              onClick={handleExport}
              variant="secondary"
              size="sm"
              disabled={entries.length === 0}
            >
              <Download className="h-4 w-4" />
              {t("vocabulary.export")}
            </Button>
          </div>
        </div>
      </SettingsGroup>

      {/* Vocabulary list */}
      <SettingsGroup title={t("vocabulary.listTitle")}>
        <div className="p-4">
          {isLoading ? (
            <p className="text-sm text-mid-gray">{t("vocabulary.loading")}</p>
          ) : filteredEntries.length === 0 ? (
            <div className="flex flex-col items-center gap-3 py-8 text-center">
              <BookA className="h-8 w-8 text-mid-gray/50" />
              <p className="text-sm text-mid-gray">
                {searchQuery.trim()
                  ? t("vocabulary.noResults")
                  : t("vocabulary.empty")}
              </p>
            </div>
          ) : (
            <div className="space-y-2 max-h-96 overflow-y-auto">
              {filteredEntries.map((entry) => (
                <div
                  key={entry.id}
                  className="flex items-center gap-3 p-3 bg-mid-gray/5 rounded border border-mid-gray/20"
                >
                  <div className="flex-1 min-w-0 flex items-center gap-2 flex-wrap">
                    <span className="text-sm font-medium truncate">
                      {entry.term}
                    </span>
                    {entry.category && (
                      <CategoryBadge category={entry.category} />
                    )}
                    <SourceBadge source={entry.source} />
                  </div>
                  <span className="text-xs text-mid-gray whitespace-nowrap">
                    {t("vocabulary.frequency", { count: entry.frequency })}
                  </span>
                  <button
                    onClick={() => handleRemoveTerm(entry.id)}
                    className="p-1 hover:bg-red-500/20 rounded transition-colors text-mid-gray hover:text-red-500"
                    aria-label={t("vocabulary.delete")}
                  >
                    <Trash2 className="h-4 w-4" />
                  </button>
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Stats footer */}
        {entries.length > 0 && (
          <div className="px-4 py-3 border-t border-mid-gray/20">
            <p className="text-xs text-mid-gray">
              {t("vocabulary.stats", { count: entries.length })}
            </p>
          </div>
        )}
      </SettingsGroup>
    </div>
  );
};
