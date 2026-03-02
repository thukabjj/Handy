import React, { useEffect, useState, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { RefreshCcw } from "lucide-react";

import { Dropdown, SettingContainer } from "@/components/ui";
import { ResetButton } from "../../ui/ResetButton";
import { Input } from "../../ui/Input";
import { OllamaConnectionStatus } from "./OllamaConnectionStatus";

interface OllamaModel {
  name: string;
  modified_at: string;
  size: number;
}

interface OllamaConfigProps {
  /** Current base URL value */
  baseUrl: string;
  /** Current model value */
  model: string;
  /** Callback when base URL changes */
  onBaseUrlChange: (url: string) => Promise<void>;
  /** Callback when model changes */
  onModelChange: (model: string) => Promise<void>;
  /** Optional translation key prefix (defaults to "shared.ollama") */
  translationPrefix?: string;
  /** Whether to show in grouped mode */
  grouped?: boolean;
}

/**
 * Shared Ollama configuration component.
 * Includes connection status, base URL input, and model selector.
 */
export const OllamaConfig: React.FC<OllamaConfigProps> = ({
  baseUrl,
  model,
  onBaseUrlChange,
  onModelChange,
  translationPrefix = "shared.ollama",
  grouped = true,
}) => {
  const { t } = useTranslation();
  const [modelOptions, setModelOptions] = useState<string[]>([]);
  const [isFetchingModels, setIsFetchingModels] = useState(false);
  const [localBaseUrl, setLocalBaseUrl] = useState(baseUrl);

  const fetchModels = useCallback(async () => {
    setIsFetchingModels(true);
    try {
      // Fetch models directly from the Ollama API
      const response = await fetch(`${localBaseUrl}/api/tags`, {
        method: "GET",
        signal: AbortSignal.timeout(10000),
      });
      if (response.ok) {
        const data = await response.json();
        const models = data.models as OllamaModel[];
        setModelOptions(models.map((m: OllamaModel) => m.name));
      }
    } catch (error) {
      console.error("Failed to fetch Ollama models:", error);
      setModelOptions([]);
    } finally {
      setIsFetchingModels(false);
    }
  }, [localBaseUrl]);

  useEffect(() => {
    fetchModels();
  }, [fetchModels]);

  useEffect(() => {
    setLocalBaseUrl(baseUrl);
  }, [baseUrl]);

  const handleBaseUrlBlur = async () => {
    if (localBaseUrl !== baseUrl) {
      await onBaseUrlChange(localBaseUrl);
      fetchModels();
    }
  };

  const handleModelChange = async (newModel: string | null) => {
    if (newModel) {
      await onModelChange(newModel);
    }
  };

  return (
    <>
      <OllamaConnectionStatus endpoint={localBaseUrl} />

      <SettingContainer
        title={t(`${translationPrefix}.baseUrl.title`)}
        description={t(`${translationPrefix}.baseUrl.description`)}
        descriptionMode="tooltip"
        layout="horizontal"
        grouped={grouped}
      >
        <Input
          value={localBaseUrl}
          onChange={(e) => setLocalBaseUrl(e.target.value)}
          onBlur={handleBaseUrlBlur}
          placeholder="http://localhost:11434"
          className="min-w-[300px]"
        />
      </SettingContainer>

      <SettingContainer
        title={t(`${translationPrefix}.model.title`)}
        description={t(`${translationPrefix}.model.description`)}
        descriptionMode="tooltip"
        layout="horizontal"
        grouped={grouped}
      >
        <div className="flex items-center gap-2">
          <Dropdown
            selectedValue={model || null}
            options={modelOptions.map((m) => ({ value: m, label: m }))}
            onSelect={handleModelChange}
            placeholder={
              modelOptions.length > 0
                ? t(`${translationPrefix}.model.placeholder`)
                : t(`${translationPrefix}.model.placeholderNoModels`)
            }
            disabled={isFetchingModels}
            className="min-w-[250px]"
          />
          <ResetButton
            onClick={fetchModels}
            disabled={isFetchingModels}
            ariaLabel={t(`${translationPrefix}.model.refresh`)}
            className="flex h-10 w-10 items-center justify-center"
          >
            <RefreshCcw
              className={`h-4 w-4 ${isFetchingModels ? "animate-spin" : ""}`}
            />
          </ResetButton>
        </div>
      </SettingContainer>
    </>
  );
};
