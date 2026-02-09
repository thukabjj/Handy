import React, { useEffect, useState, useCallback } from "react";
import { useTranslation } from "react-i18next";
import {
  RefreshCcw,
  Database,
  Trash2,
  FileText,
  Upload,
  Search,
} from "lucide-react";
import { commands, StoredDocument, RagStats, SearchResult } from "@/bindings";

import {
  Dropdown,
  SettingContainer,
  SettingsGroup,
  ToggleSwitch,
  Slider,
  Textarea,
} from "@/components/ui";
import { Button } from "../../ui/Button";
import { Input } from "../../ui/Input";
import { useSettings } from "../../../hooks/useSettings";

const DisabledNotice: React.FC<{ children: React.ReactNode }> = ({
  children,
}) => (
  <div className="p-4 bg-mid-gray/5 rounded-lg border border-mid-gray/20">
    <p className="text-sm text-mid-gray">{children}</p>
  </div>
);

const StatsDisplay: React.FC = () => {
  const { t } = useTranslation();
  const [stats, setStats] = useState<RagStats | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  const fetchStats = useCallback(async () => {
    setIsLoading(true);
    try {
      const result = await commands.ragGetStats();
      if (result.status === "ok") {
        setStats(result.data);
      }
    } catch (error) {
      console.error("Failed to fetch RAG stats:", error);
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchStats();
  }, [fetchStats]);

  return (
    <div className="flex items-center gap-4 p-3 bg-mid-gray/5 rounded-lg border border-mid-gray/20">
      <Database className="h-5 w-5 text-mid-gray" />
      <div className="flex-1">
        <div className="flex items-center gap-4">
          <span className="text-sm">
            <span className="font-medium">{stats?.document_count ?? 0}</span>{" "}
            {t("settings.knowledgeBase.stats.documents")}
          </span>
          <span className="text-sm">
            <span className="font-medium">{stats?.embedding_count ?? 0}</span>{" "}
            {t("settings.knowledgeBase.stats.embeddings")}
          </span>
        </div>
      </div>
      <button
        onClick={fetchStats}
        disabled={isLoading}
        className="p-1 hover:bg-mid-gray/20 rounded transition-colors"
        title={t("settings.knowledgeBase.stats.refresh")}
      >
        <RefreshCcw className={`h-4 w-4 ${isLoading ? "animate-spin" : ""}`} />
      </button>
    </div>
  );
};

const EmbeddingModelSelector: React.FC = () => {
  const { t } = useTranslation();
  const { getSetting, refreshSettings } = useSettings();
  const [modelOptions, setModelOptions] = useState<string[]>([]);
  const [isFetchingModels, setIsFetchingModels] = useState(false);

  const knowledgeBase = getSetting("knowledge_base");
  const model = knowledgeBase?.embedding_model ?? "nomic-embed-text";

  const fetchModels = useCallback(async () => {
    setIsFetchingModels(true);
    try {
      const result = await commands.fetchOllamaModels();
      if (result.status === "ok") {
        // Filter to show only embedding models (common patterns)
        const embeddingPatterns = [
          "nomic-embed",
          "all-minilm",
          "bge-",
          "gte-",
          "e5-",
          "embed",
        ];
        const allModels = result.data.map((m) => m.name);
        // Show embedding models first, then all others
        const embeddingModels = allModels.filter((m) =>
          embeddingPatterns.some((p) => m.toLowerCase().includes(p)),
        );
        const otherModels = allModels.filter(
          (m) => !embeddingPatterns.some((p) => m.toLowerCase().includes(p)),
        );
        setModelOptions([...embeddingModels, ...otherModels]);
      }
    } catch (error) {
      console.error("Failed to fetch Ollama models:", error);
    } finally {
      setIsFetchingModels(false);
    }
  }, []);

  useEffect(() => {
    fetchModels();
  }, [fetchModels]);

  const handleModelChange = async (newModel: string | null) => {
    if (!newModel) return;
    const result = await commands.changeKbEmbeddingModelSetting(newModel);
    if (result.status === "ok") {
      await refreshSettings();
    }
  };

  return (
    <SettingContainer
      title={t("settings.knowledgeBase.embeddingModel.title")}
      description={t("settings.knowledgeBase.embeddingModel.description")}
      descriptionMode="tooltip"
      layout="horizontal"
      grouped={true}
    >
      <div className="flex items-center gap-2">
        <Dropdown
          selectedValue={model}
          options={modelOptions.map((m) => ({ value: m, label: m }))}
          onSelect={handleModelChange}
          placeholder={
            modelOptions.length > 0
              ? t("settings.knowledgeBase.embeddingModel.placeholder")
              : t("settings.knowledgeBase.embeddingModel.placeholderNoModels")
          }
          disabled={isFetchingModels}
          className="min-w-[250px]"
        />
        <button
          onClick={fetchModels}
          disabled={isFetchingModels}
          className="p-2 hover:bg-mid-gray/20 rounded transition-colors"
          title={t("settings.knowledgeBase.embeddingModel.refresh")}
        >
          <RefreshCcw
            className={`h-4 w-4 ${isFetchingModels ? "animate-spin" : ""}`}
          />
        </button>
      </div>
    </SettingContainer>
  );
};

const RetrievalSettings: React.FC = () => {
  const { t } = useTranslation();
  const { getSetting, refreshSettings } = useSettings();

  const knowledgeBase = getSetting("knowledge_base");
  const topK = knowledgeBase?.top_k ?? 3;
  const similarityThreshold = knowledgeBase?.similarity_threshold ?? 0.5;

  const handleTopKChange = async (value: string | null) => {
    if (!value) return;
    const result = await commands.changeKbTopKSetting(parseInt(value));
    if (result.status === "ok") {
      await refreshSettings();
    }
  };

  const handleThresholdChange = async (value: number) => {
    const result = await commands.changeKbSimilarityThresholdSetting(value);
    if (result.status === "ok") {
      await refreshSettings();
    }
  };

  return (
    <>
      <SettingContainer
        title={t("settings.knowledgeBase.topK.title")}
        description={t("settings.knowledgeBase.topK.description")}
        descriptionMode="tooltip"
        layout="horizontal"
        grouped={true}
      >
        <Dropdown
          selectedValue={topK.toString()}
          options={[
            { value: "1", label: "1" },
            { value: "2", label: "2" },
            { value: "3", label: "3" },
            { value: "5", label: "5" },
            { value: "10", label: "10" },
          ]}
          onSelect={handleTopKChange}
        />
      </SettingContainer>

      <Slider
        label={t("settings.knowledgeBase.similarityThreshold.title")}
        description={t(
          "settings.knowledgeBase.similarityThreshold.description",
        )}
        descriptionMode="tooltip"
        grouped={true}
        min={0}
        max={1}
        step={0.05}
        value={similarityThreshold}
        onChange={handleThresholdChange}
        formatValue={(v) => `${(v * 100).toFixed(0)}%`}
      />
    </>
  );
};

const DocumentBrowser: React.FC = () => {
  const { t } = useTranslation();
  const [documents, setDocuments] = useState<StoredDocument[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [searchQuery, setSearchQuery] = useState("");
  const [searchResults, setSearchResults] = useState<SearchResult[] | null>(
    null,
  );
  const [isSearching, setIsSearching] = useState(false);

  const fetchDocuments = useCallback(async () => {
    setIsLoading(true);
    try {
      const result = await commands.ragListDocuments();
      if (result.status === "ok") {
        setDocuments(result.data);
      }
    } catch (error) {
      console.error("Failed to fetch documents:", error);
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchDocuments();
  }, [fetchDocuments]);

  const handleDelete = async (documentId: number) => {
    const result = await commands.ragDeleteDocument(documentId);
    if (result.status === "ok") {
      await fetchDocuments();
    }
  };

  const handleClearAll = async () => {
    if (
      !window.confirm(t("settings.knowledgeBase.documents.confirmClearAll"))
    ) {
      return;
    }
    const result = await commands.ragClearAll();
    if (result.status === "ok") {
      await fetchDocuments();
      setSearchResults(null);
    }
  };

  const handleSearch = async () => {
    if (!searchQuery.trim()) {
      setSearchResults(null);
      return;
    }
    setIsSearching(true);
    try {
      const result = await commands.ragSearch(searchQuery, null);
      if (result.status === "ok") {
        setSearchResults(result.data);
      }
    } catch (error) {
      console.error("Failed to search:", error);
    } finally {
      setIsSearching(false);
    }
  };

  const formatDate = (timestamp: number) => {
    return new Date(timestamp * 1000).toLocaleDateString(undefined, {
      year: "numeric",
      month: "short",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  };

  return (
    <SettingsGroup title={t("settings.knowledgeBase.documents.title")}>
      {/* Search bar */}
      <div className="flex gap-2 mb-4">
        <Input
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          placeholder={t("settings.knowledgeBase.documents.searchPlaceholder")}
          className="flex-1"
          onKeyDown={(e) => e.key === "Enter" && handleSearch()}
        />
        <Button
          onClick={handleSearch}
          variant="secondary"
          size="md"
          disabled={isSearching}
        >
          <Search className={`h-4 w-4 ${isSearching ? "animate-spin" : ""}`} />
        </Button>
      </div>

      {/* Search results */}
      {searchResults !== null && (
        <div className="mb-4">
          <div className="flex items-center justify-between mb-2">
            <h4 className="text-sm font-medium">
              {t("settings.knowledgeBase.documents.searchResults", {
                count: searchResults.length,
              })}
            </h4>
            <button
              onClick={() => {
                setSearchResults(null);
                setSearchQuery("");
              }}
              className="text-xs text-mid-gray hover:text-text"
            >
              {t("settings.knowledgeBase.documents.clearSearch")}
            </button>
          </div>
          {searchResults.length > 0 ? (
            <div className="space-y-2 max-h-60 overflow-y-auto">
              {searchResults.map((result, idx) => (
                <div
                  key={idx}
                  className="p-3 bg-mid-gray/5 rounded border border-mid-gray/20"
                >
                  <div className="flex items-center justify-between mb-1">
                    <span className="text-sm font-medium">
                      {result.title ||
                        t("settings.knowledgeBase.documents.untitled")}
                    </span>
                    <span className="text-xs text-mid-gray">
                      {(result.similarity * 100).toFixed(1)}%{" "}
                      {t("settings.knowledgeBase.documents.match")}
                    </span>
                  </div>
                  <p className="text-xs text-mid-gray line-clamp-2">
                    {result.chunk_text}
                  </p>
                </div>
              ))}
            </div>
          ) : (
            <p className="text-sm text-mid-gray">
              {t("settings.knowledgeBase.documents.noResults")}
            </p>
          )}
        </div>
      )}

      {/* Document list */}
      <div className="flex items-center justify-between mb-2">
        <h4 className="text-sm font-medium">
          {t("settings.knowledgeBase.documents.allDocuments")} (
          {documents.length})
        </h4>
        {documents.length > 0 && (
          <Button onClick={handleClearAll} variant="secondary" size="sm">
            <Trash2 className="h-3 w-3 mr-1" />
            {t("settings.knowledgeBase.documents.clearAll")}
          </Button>
        )}
      </div>

      {isLoading ? (
        <p className="text-sm text-mid-gray">
          {t("settings.knowledgeBase.documents.loading")}
        </p>
      ) : documents.length === 0 ? (
        <p className="text-sm text-mid-gray">
          {t("settings.knowledgeBase.documents.empty")}
        </p>
      ) : (
        <div className="space-y-2 max-h-80 overflow-y-auto">
          {documents.map((doc) => (
            <div
              key={doc.id}
              className="flex items-start gap-3 p-3 bg-mid-gray/5 rounded border border-mid-gray/20"
            >
              <FileText className="h-4 w-4 text-mid-gray mt-0.5 shrink-0" />
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2">
                  <span className="text-sm font-medium truncate">
                    {doc.title ||
                      t("settings.knowledgeBase.documents.untitled")}
                  </span>
                  <span className="text-xs px-1.5 py-0.5 bg-mid-gray/20 rounded">
                    {doc.source_type}
                  </span>
                </div>
                <p className="text-xs text-mid-gray mt-0.5">
                  {formatDate(doc.created_at)}
                </p>
                <p className="text-xs text-mid-gray/70 mt-1 line-clamp-2">
                  {doc.content.slice(0, 150)}
                  {doc.content.length > 150 ? "..." : ""}
                </p>
              </div>
              <button
                onClick={() => handleDelete(doc.id)}
                className="p-1 hover:bg-red-500/20 rounded transition-colors text-mid-gray hover:text-red-500"
                title={t("settings.knowledgeBase.documents.delete")}
              >
                <Trash2 className="h-4 w-4" />
              </button>
            </div>
          ))}
        </div>
      )}
    </SettingsGroup>
  );
};

const ManualDocumentUpload: React.FC = () => {
  const { t } = useTranslation();
  const [content, setContent] = useState("");
  const [title, setTitle] = useState("");
  const [isUploading, setIsUploading] = useState(false);

  const handleUpload = async () => {
    if (!content.trim()) return;
    setIsUploading(true);
    try {
      const result = await commands.ragAddDocument(
        content.trim(),
        "manual",
        null,
        title.trim() || null,
      );
      if (result.status === "ok") {
        setContent("");
        setTitle("");
      }
    } catch (error) {
      console.error("Failed to add document:", error);
    } finally {
      setIsUploading(false);
    }
  };

  return (
    <SettingsGroup title={t("settings.knowledgeBase.upload.title")}>
      <div className="space-y-3">
        <Input
          value={title}
          onChange={(e) => setTitle(e.target.value)}
          placeholder={t("settings.knowledgeBase.upload.titlePlaceholder")}
        />
        <Textarea
          value={content}
          onChange={(e) => setContent(e.target.value)}
          placeholder={t("settings.knowledgeBase.upload.contentPlaceholder")}
          rows={4}
        />
        <div className="flex justify-end">
          <Button
            onClick={handleUpload}
            variant="primary"
            size="md"
            disabled={!content.trim() || isUploading}
          >
            <Upload className="h-4 w-4 mr-2" />
            {isUploading
              ? t("settings.knowledgeBase.upload.uploading")
              : t("settings.knowledgeBase.upload.addDocument")}
          </Button>
        </div>
      </div>
    </SettingsGroup>
  );
};

export const KnowledgeBaseSettings: React.FC = () => {
  const { t } = useTranslation();
  const { getSetting, refreshSettings } = useSettings();

  const knowledgeBase = getSetting("knowledge_base");
  const enabled = knowledgeBase?.enabled ?? false;
  const autoIndex = knowledgeBase?.auto_index_transcriptions ?? true;
  const useInActiveListening = knowledgeBase?.use_in_active_listening ?? true;

  const handleEnableChange = async (value: boolean) => {
    const result = await commands.changeKnowledgeBaseEnabledSetting(value);
    if (result.status === "ok") {
      await refreshSettings();
    }
  };

  const handleAutoIndexChange = async (value: boolean) => {
    const result = await commands.changeAutoIndexTranscriptionsSetting(value);
    if (result.status === "ok") {
      await refreshSettings();
    }
  };

  const handleUseInActiveListeningChange = async (value: boolean) => {
    const result = await commands.changeKbUseInActiveListeningSetting(value);
    if (result.status === "ok") {
      await refreshSettings();
    }
  };

  return (
    <div className="max-w-3xl w-full mx-auto space-y-6">
      <SettingsGroup title={t("settings.knowledgeBase.general.title")}>
        <ToggleSwitch
          label={t("settings.knowledgeBase.enable.title")}
          description={t("settings.knowledgeBase.enable.description")}
          checked={enabled}
          onChange={handleEnableChange}
          grouped={true}
        />
      </SettingsGroup>

      {enabled && (
        <>
          <SettingsGroup title={t("settings.knowledgeBase.stats.title")}>
            <StatsDisplay />
          </SettingsGroup>

          <SettingsGroup title={t("settings.knowledgeBase.indexing.title")}>
            <ToggleSwitch
              label={t("settings.knowledgeBase.autoIndex.title")}
              description={t("settings.knowledgeBase.autoIndex.description")}
              checked={autoIndex}
              onChange={handleAutoIndexChange}
              grouped={true}
            />
            <ToggleSwitch
              label={t("settings.knowledgeBase.useInActiveListening.title")}
              description={t(
                "settings.knowledgeBase.useInActiveListening.description",
              )}
              checked={useInActiveListening}
              onChange={handleUseInActiveListeningChange}
              grouped={true}
            />
          </SettingsGroup>

          <SettingsGroup
            title={t("settings.knowledgeBase.configuration.title")}
          >
            <EmbeddingModelSelector />
            <RetrievalSettings />
          </SettingsGroup>

          <DocumentBrowser />
          <ManualDocumentUpload />
        </>
      )}

      {!enabled && (
        <DisabledNotice>
          {t("settings.knowledgeBase.disabledNotice")}
        </DisabledNotice>
      )}
    </div>
  );
};
