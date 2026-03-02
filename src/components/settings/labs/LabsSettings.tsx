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

  // Get feature states - use settings object directly for type safety
  const postProcessEnabled = settings?.post_process_enabled ?? false;
  const experimentalEnabled = settings?.experimental_enabled ?? false;

  // Toggle handlers using updateSetting for features stored directly
  const handlePostProcessingToggle = async (value: boolean) => {
    await commands.changePostProcessEnabledSetting(value);
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
        {postProcessEnabled && <FeatureConfigHint featureKey="postProcessing" />}
      </FeatureCard>

      {/* Coming Soon Section - Features requiring backend updates */}
      <div className="space-y-4 opacity-60">
        <p className="text-xs text-mid-gray uppercase tracking-wider px-1">
          {t("labs.comingSoon", "Coming Soon")}
        </p>

        {/* Active Listening - Placeholder */}
        <FeatureCard
          title={t("labs.activeListening.title")}
          description={t("labs.activeListening.description")}
          enabled={false}
          onToggle={() => {}}
          badge="beta"
          icon={<Headphones className="h-5 w-5" />}
          disabled
        />

        {/* Ask AI - Placeholder */}
        <FeatureCard
          title={t("labs.askAi.title")}
          description={t("labs.askAi.description")}
          enabled={false}
          onToggle={() => {}}
          badge="beta"
          icon={<MessageSquare className="h-5 w-5" />}
          disabled
        />

        {/* Knowledge Base - Placeholder */}
        <FeatureCard
          title={t("labs.knowledgeBase.title")}
          description={t("labs.knowledgeBase.description")}
          enabled={false}
          onToggle={() => {}}
          badge="beta"
          icon={<BookOpen className="h-5 w-5" />}
          disabled
        />
      </div>
    </div>
  );
};

/**
 * Shows configuration status for a Labs feature.
 */
interface FeatureConfigHintProps {
  ollamaModel?: string;
  embeddingModel?: string;
  featureKey: string;
}

const FeatureConfigHint: React.FC<FeatureConfigHintProps> = ({
  ollamaModel,
  embeddingModel,
}) => {
  const { t } = useTranslation();

  const hasOllamaConfig = ollamaModel && ollamaModel.length > 0;
  const hasEmbeddingConfig = embeddingModel && embeddingModel.length > 0;
  const isConfigured = hasOllamaConfig || hasEmbeddingConfig;

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
                {hasOllamaConfig && (
                  <span>
                    {t("labs.configuredWith", "Using")}{" "}
                    <code className="text-xs bg-mid-gray/20 px-1 py-0.5 rounded">
                      {ollamaModel}
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
              t("labs.notConfigured", "Not configured - requires Ollama")
            )}
          </span>
        </div>
      </div>
    </div>
  );
};
