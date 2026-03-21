import React from "react";
import { useTranslation } from "react-i18next";
import {
  Headphones,
  MessageSquare,
  BookOpen,
  Sparkles,
  FlaskConical,
} from "lucide-react";
import { commands } from "@/bindings";
import { useSettings } from "../../../hooks/useSettings";
import { FeatureCard } from "./FeatureCard";

/**
 * LabsSettings - Experimental features with progressive disclosure.
 * Features are hidden by default and show a "Configure" link when enabled.
 */
export const LabsSettings: React.FC = () => {
  const { t } = useTranslation();
  const { settings, updateSetting, refreshSettings } = useSettings();

  // Get feature states
  const postProcessEnabled = settings?.post_process_enabled ?? false;
  const activeListeningEnabled = settings?.active_listening?.enabled ?? false;
  const askAiEnabled = settings?.ask_ai?.enabled ?? false;
  const knowledgeBaseEnabled = settings?.knowledge_base?.enabled ?? false;

  // Toggle handlers
  const handlePostProcessingToggle = async (value: boolean) => {
    await commands.changePostProcessEnabledSetting(value);
    await refreshSettings();
  };

  const handleActiveListeningToggle = async (value: boolean) => {
    await commands.changeActiveListeningEnabledSetting(value);
    await refreshSettings();
  };

  const handleAskAiToggle = async (value: boolean) => {
    await commands.changeAskAiEnabledSetting(value);
    await refreshSettings();
  };

  const handleKnowledgeBaseToggle = async (value: boolean) => {
    await commands.changeKnowledgeBaseEnabledSetting(value);
    await refreshSettings();
  };

  return (
    <div className="max-w-3xl w-full mx-auto space-y-6">
      {/* Labs Header */}
      <div className="flex items-center gap-3 p-4 bg-purple-500/10 rounded-lg border border-purple-500/20">
        <FlaskConical className="h-5 w-5 text-purple-400" />
        <div>
          <h2 className="font-medium text-text">{t("labs.title")}</h2>
          <p className="text-sm text-mid-gray">{t("labs.description")}</p>
        </div>
      </div>

      {/* Post-Processing - Main experimental feature that works */}
      <FeatureCard
        title={t("labs.postProcessing.title")}
        description={t("labs.postProcessing.description")}
        enabled={postProcessEnabled}
        onToggle={handlePostProcessingToggle}
        icon={<Sparkles className="h-5 w-5" />}
      >
        {postProcessEnabled && (
          <FeatureConfigHint featureKey="postProcessing" />
        )}
      </FeatureCard>

      {/* Active Listening */}
      <FeatureCard
        title={t("labs.activeListening.title")}
        description={t("labs.activeListening.description")}
        enabled={activeListeningEnabled}
        onToggle={handleActiveListeningToggle}
        badge="beta"
        icon={<Headphones className="h-5 w-5" />}
      >
        {activeListeningEnabled && (
          <FeatureConfigHint
            featureKey="activeListening"
            model={
              settings?.active_listening?.llm_provider === "ollama"
                ? settings?.active_listening?.ollama_model
                : (settings?.active_listening?.llm_model ?? undefined)
            }
          />
        )}
      </FeatureCard>

      {/* Ask AI */}
      <FeatureCard
        title={t("labs.askAi.title")}
        description={t("labs.askAi.description")}
        enabled={askAiEnabled}
        onToggle={handleAskAiToggle}
        badge="beta"
        icon={<MessageSquare className="h-5 w-5" />}
      >
        {askAiEnabled && (
          <FeatureConfigHint
            featureKey="askAi"
            model={
              settings?.active_listening?.llm_provider === "ollama"
                ? settings?.active_listening?.ollama_model
                : (settings?.active_listening?.llm_model ?? undefined)
            }
          />
        )}
      </FeatureCard>

      {/* Knowledge Base */}
      <FeatureCard
        title={t("labs.knowledgeBase.title")}
        description={t("labs.knowledgeBase.description")}
        enabled={knowledgeBaseEnabled}
        onToggle={handleKnowledgeBaseToggle}
        badge="beta"
        icon={<BookOpen className="h-5 w-5" />}
      >
        {knowledgeBaseEnabled && (
          <FeatureConfigHint
            featureKey="knowledgeBase"
            embeddingModel={
              settings?.knowledge_base?.embedding_model ?? undefined
            }
          />
        )}
      </FeatureCard>
    </div>
  );
};

/**
 * Shows configuration status for a Labs feature.
 */
interface FeatureConfigHintProps {
  model?: string;
  embeddingModel?: string;
  featureKey: string;
}

const FeatureConfigHint: React.FC<FeatureConfigHintProps> = ({
  model,
  embeddingModel,
}) => {
  const { t } = useTranslation();

  const hasModelConfig = model && model.length > 0;
  const hasEmbeddingConfig = embeddingModel && embeddingModel.length > 0;
  const isConfigured = hasModelConfig || hasEmbeddingConfig;

  return (
    <div className="mt-3 pt-3 border-t border-mid-gray/20">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <div
            className={`w-2 h-2 rounded-full ${isConfigured ? "bg-green-500" : "bg-yellow-500"}`}
          />
          <span className="text-sm text-mid-gray">
            {isConfigured ? (
              <>
                {hasModelConfig && (
                  <span>
                    {t("labs.configuredWith", "Using")}{" "}
                    <code className="text-xs bg-mid-gray/20 px-1 py-0.5 rounded">
                      {model}
                    </code>
                  </span>
                )}
                {hasEmbeddingConfig && (
                  <span>
                    {t("labs.configuredWith", "Using")}{" "}
                    <code className="text-xs bg-mid-gray/20 px-1 py-0.5 rounded">
                      {embeddingModel}
                    </code>
                  </span>
                )}
              </>
            ) : (
              t(
                "labs.notConfigured",
                "Not configured - requires an AI provider",
              )
            )}
          </span>
        </div>
      </div>
    </div>
  );
};
