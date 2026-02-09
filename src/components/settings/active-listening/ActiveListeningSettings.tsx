import React, { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { RefreshCcw, Wifi, WifiOff } from "lucide-react";
import { commands } from "@/bindings";

import {
  Dropdown,
  SettingContainer,
  SettingsGroup,
  ToggleSwitch,
  Textarea,
} from "@/components/ui";
import { Button } from "../../ui/Button";
import { ResetButton } from "../../ui/ResetButton";
import { Input } from "../../ui/Input";
import { useSettings } from "../../../hooks/useSettings";
import { HandyShortcut } from "../HandyShortcut";
import { SessionViewer } from "./SessionViewer";
import { AudioSourceSettings } from "./AudioSourceSettings";

const DisabledNotice: React.FC<{ children: React.ReactNode }> = ({
  children,
}) => (
  <div className="p-4 bg-mid-gray/5 rounded-lg border border-mid-gray/20">
    <p className="text-sm text-mid-gray">{children}</p>
  </div>
);

const OllamaConnectionStatus: React.FC = () => {
  const { t } = useTranslation();
  const [isConnected, setIsConnected] = useState<boolean | null>(null);
  const [isChecking, setIsChecking] = useState(false);

  const checkConnection = async () => {
    setIsChecking(true);
    try {
      const result = await commands.checkOllamaConnection();
      if (result.status === "ok") {
        setIsConnected(result.data);
      } else {
        setIsConnected(false);
      }
    } catch {
      setIsConnected(false);
    } finally {
      setIsChecking(false);
    }
  };

  useEffect(() => {
    checkConnection();
  }, []);

  const statusText = isChecking
    ? t("settings.activeListening.ollama.connectionStatus.checking")
    : isConnected
      ? t("settings.activeListening.ollama.connectionStatus.connected")
      : t("settings.activeListening.ollama.connectionStatus.disconnected");

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
        title={t("settings.activeListening.ollama.refreshConnection")}
        aria-label={t("settings.activeListening.ollama.refreshConnection")}
      >
        <RefreshCcw className={`h-4 w-4 ${isChecking ? "animate-spin" : ""}`} />
      </button>
    </div>
  );
};

const OllamaSettingsComponent: React.FC = () => {
  const { t } = useTranslation();
  const { getSetting, updateSetting, isUpdating, refreshSettings } =
    useSettings();
  const [modelOptions, setModelOptions] = useState<string[]>([]);
  const [isFetchingModels, setIsFetchingModels] = useState(false);

  const activeListening = getSetting("active_listening");
  const baseUrl = activeListening?.ollama_base_url ?? "http://localhost:11434";
  const model = activeListening?.ollama_model ?? "";

  const fetchModels = async () => {
    setIsFetchingModels(true);
    try {
      const result = await commands.fetchOllamaModels();
      if (result.status === "ok") {
        setModelOptions(result.data.map((m) => m.name));
      }
    } catch (error) {
      console.error("Failed to fetch Ollama models:", error);
    } finally {
      setIsFetchingModels(false);
    }
  };

  useEffect(() => {
    fetchModels();
  }, [baseUrl]);

  const handleBaseUrlChange = async (newUrl: string) => {
    if (!activeListening) return;
    await commands.changeOllamaBaseUrlSetting(newUrl);
    await refreshSettings();
    fetchModels();
  };

  const handleModelChange = async (newModel: string | null) => {
    if (!activeListening || !newModel) return;
    await commands.changeOllamaModelSetting(newModel);
    await refreshSettings();
  };

  return (
    <>
      <OllamaConnectionStatus />

      <SettingContainer
        title={t("settings.activeListening.ollama.baseUrl.title")}
        description={t("settings.activeListening.ollama.baseUrl.description")}
        descriptionMode="tooltip"
        layout="horizontal"
        grouped={true}
      >
        <Input
          value={baseUrl}
          onBlur={(e) => handleBaseUrlChange(e.target.value)}
          placeholder="http://localhost:11434"
          className="min-w-[300px]"
        />
      </SettingContainer>

      <SettingContainer
        title={t("settings.activeListening.ollama.model.title")}
        description={t("settings.activeListening.ollama.model.description")}
        descriptionMode="tooltip"
        layout="horizontal"
        grouped={true}
      >
        <div className="flex items-center gap-2">
          <Dropdown
            selectedValue={model || null}
            options={modelOptions.map((m) => ({ value: m, label: m }))}
            onSelect={handleModelChange}
            placeholder={
              modelOptions.length > 0
                ? t("settings.activeListening.ollama.model.placeholder")
                : t("settings.activeListening.ollama.model.placeholderNoModels")
            }
            disabled={isFetchingModels}
            className="min-w-[250px]"
          />
          <ResetButton
            onClick={fetchModels}
            disabled={isFetchingModels}
            ariaLabel={t("settings.activeListening.ollama.model.refresh")}
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

const SegmentSettingsComponent: React.FC = () => {
  const { t } = useTranslation();
  const { getSetting, refreshSettings } = useSettings();

  const activeListening = getSetting("active_listening");
  const segmentDuration =
    activeListening?.segment_duration_seconds?.toString() ?? "15";

  const handleDurationChange = async (value: string | null) => {
    if (!value) return;
    await commands.changeActiveListeningSegmentDurationSetting(parseInt(value));
    await refreshSettings();
  };

  return (
    <SettingContainer
      title={t("settings.activeListening.segments.duration.title")}
      description={t("settings.activeListening.segments.duration.description")}
      descriptionMode="tooltip"
      layout="horizontal"
      grouped={true}
    >
      <Dropdown
        selectedValue={segmentDuration}
        options={[
          {
            value: "10",
            label: t("settings.activeListening.segments.duration.seconds", {
              count: 10,
            }),
          },
          {
            value: "15",
            label: t("settings.activeListening.segments.duration.seconds", {
              count: 15,
            }),
          },
          {
            value: "20",
            label: t("settings.activeListening.segments.duration.seconds", {
              count: 20,
            }),
          },
          {
            value: "30",
            label: t("settings.activeListening.segments.duration.seconds", {
              count: 30,
            }),
          },
        ]}
        onSelect={handleDurationChange}
      />
    </SettingContainer>
  );
};

// Helper to group prompts by category
type PromptCategory = "note_taking" | "meeting_coach" | "custom";

const getCategoryLabel = (
  category: PromptCategory,
  t: ReturnType<typeof useTranslation>["t"],
): string => {
  return t(`settings.activeListening.prompts.categories.${category}`);
};

const getCategoryOrder = (category: PromptCategory): number => {
  switch (category) {
    case "meeting_coach":
      return 0; // Meeting Coach first (most relevant for real-time assistance)
    case "note_taking":
      return 1;
    case "custom":
      return 2;
    default:
      return 3;
  }
};

const PromptsEditorComponent: React.FC = () => {
  const { t } = useTranslation();
  const { getSetting, refreshSettings } = useSettings();
  const [isCreating, setIsCreating] = useState(false);
  const [draftName, setDraftName] = useState("");
  const [draftTemplate, setDraftTemplate] = useState("");

  const activeListening = getSetting("active_listening");
  const prompts = activeListening?.prompts ?? [];
  const selectedPromptId = activeListening?.selected_prompt_id ?? "";
  const selectedPrompt = prompts.find((p) => p.id === selectedPromptId) || null;

  // Group prompts by category for the dropdown
  const groupedOptions = React.useMemo(() => {
    const groups: Record<PromptCategory, typeof prompts> = {
      meeting_coach: [],
      note_taking: [],
      custom: [],
    };

    for (const prompt of prompts) {
      // Use 'category' field if available, fallback to heuristics
      const category: PromptCategory =
        (prompt as { category?: PromptCategory }).category ||
        (prompt.is_default
          ? prompt.id.startsWith("meeting_coach")
            ? "meeting_coach"
            : "note_taking"
          : "custom");
      groups[category].push(prompt);
    }

    return Object.entries(groups)
      .filter(([, list]) => list.length > 0)
      .sort(
        ([a], [b]) =>
          getCategoryOrder(a as PromptCategory) -
          getCategoryOrder(b as PromptCategory),
      )
      .map(([category, list]) => ({
        label: getCategoryLabel(category as PromptCategory, t),
        options: list.map((p) => ({
          value: p.id,
          label: p.name,
        })),
      }));
  }, [prompts, t]);

  useEffect(() => {
    if (isCreating) return;

    if (selectedPrompt) {
      setDraftName(selectedPrompt.name);
      setDraftTemplate(selectedPrompt.prompt_template);
    } else {
      setDraftName("");
      setDraftTemplate("");
    }
  }, [
    isCreating,
    selectedPromptId,
    selectedPrompt?.name,
    selectedPrompt?.prompt_template,
  ]);

  const handlePromptSelect = async (promptId: string | null) => {
    if (!promptId) return;
    await commands.setActiveListeningSelectedPrompt(promptId);
    await refreshSettings();
    setIsCreating(false);
  };

  const handleCreatePrompt = async () => {
    if (!draftName.trim() || !draftTemplate.trim()) return;

    try {
      const result = await commands.addActiveListeningPrompt(
        draftName.trim(),
        draftTemplate.trim(),
      );
      if (result.status === "ok") {
        await refreshSettings();
        await commands.setActiveListeningSelectedPrompt(result.data.id);
        await refreshSettings();
        setIsCreating(false);
      }
    } catch (error) {
      console.error("Failed to create prompt:", error);
    }
  };

  const handleUpdatePrompt = async () => {
    if (!selectedPromptId || !draftName.trim() || !draftTemplate.trim()) return;

    try {
      await commands.updateActiveListeningPrompt(
        selectedPromptId,
        draftName.trim(),
        draftTemplate.trim(),
      );
      await refreshSettings();
    } catch (error) {
      console.error("Failed to update prompt:", error);
    }
  };

  const handleDeletePrompt = async (promptId: string) => {
    if (!promptId) return;

    try {
      await commands.deleteActiveListeningPrompt(promptId);
      await refreshSettings();
      setIsCreating(false);
    } catch (error) {
      console.error("Failed to delete prompt:", error);
    }
  };

  const handleCancelCreate = () => {
    setIsCreating(false);
    if (selectedPrompt) {
      setDraftName(selectedPrompt.name);
      setDraftTemplate(selectedPrompt.prompt_template);
    } else {
      setDraftName("");
      setDraftTemplate("");
    }
  };

  const handleStartCreate = () => {
    setIsCreating(true);
    setDraftName("");
    setDraftTemplate("");
  };

  const hasPrompts = prompts.length > 0;
  const isDirty =
    !!selectedPrompt &&
    (draftName.trim() !== selectedPrompt.name ||
      draftTemplate.trim() !== selectedPrompt.prompt_template.trim());

  return (
    <SettingContainer
      title={t("settings.activeListening.prompts.selectedPrompt.title")}
      description={t(
        "settings.activeListening.prompts.selectedPrompt.description",
      )}
      descriptionMode="tooltip"
      layout="stacked"
      grouped={true}
    >
      <div className="space-y-3">
        <div className="flex gap-2">
          <Dropdown
            selectedValue={selectedPromptId || null}
            options={groupedOptions}
            onSelect={(value) => handlePromptSelect(value)}
            placeholder={
              prompts.length === 0
                ? t("settings.activeListening.prompts.noPrompts")
                : t("settings.activeListening.prompts.selectPrompt")
            }
            disabled={isCreating}
            className="flex-1"
          />
          <Button
            onClick={handleStartCreate}
            variant="primary"
            size="md"
            disabled={isCreating}
          >
            {t("settings.activeListening.prompts.createNew")}
          </Button>
        </div>

        {!isCreating && hasPrompts && selectedPrompt && (
          <div className="space-y-3">
            <div className="space-y-2 flex flex-col">
              <label className="text-sm font-semibold">
                {t("settings.activeListening.prompts.promptLabel")}
              </label>
              <Input
                type="text"
                value={draftName}
                onChange={(e) => setDraftName(e.target.value)}
                placeholder={t(
                  "settings.activeListening.prompts.promptLabelPlaceholder",
                )}
                variant="compact"
              />
            </div>

            <div className="space-y-2 flex flex-col">
              <label className="text-sm font-semibold">
                {t("settings.activeListening.prompts.promptTemplate")}
              </label>
              <Textarea
                value={draftTemplate}
                onChange={(e) => setDraftTemplate(e.target.value)}
                placeholder={t(
                  "settings.activeListening.prompts.promptTemplatePlaceholder",
                )}
                rows={6}
              />
              <p
                className="text-xs text-mid-gray/70"
                dangerouslySetInnerHTML={{
                  __html: t("settings.activeListening.prompts.promptTip"),
                }}
              />
            </div>

            <div className="flex gap-2 pt-2">
              <Button
                onClick={handleUpdatePrompt}
                variant="primary"
                size="md"
                disabled={
                  !draftName.trim() || !draftTemplate.trim() || !isDirty
                }
              >
                {t("settings.activeListening.prompts.updatePrompt")}
              </Button>
              <Button
                onClick={() => handleDeletePrompt(selectedPromptId)}
                variant="secondary"
                size="md"
                disabled={
                  !selectedPromptId ||
                  prompts.length <= 1 ||
                  selectedPrompt?.is_default
                }
              >
                {t("settings.activeListening.prompts.deletePrompt")}
              </Button>
            </div>
          </div>
        )}

        {!isCreating && !selectedPrompt && (
          <div className="p-3 bg-mid-gray/5 rounded border border-mid-gray/20">
            <p className="text-sm text-mid-gray">
              {hasPrompts
                ? t("settings.activeListening.prompts.selectToEdit")
                : t("settings.activeListening.prompts.createFirst")}
            </p>
          </div>
        )}

        {isCreating && (
          <div className="space-y-3">
            <div className="space-y-2 block flex flex-col">
              <label className="text-sm font-semibold text-text">
                {t("settings.activeListening.prompts.promptLabel")}
              </label>
              <Input
                type="text"
                value={draftName}
                onChange={(e) => setDraftName(e.target.value)}
                placeholder={t(
                  "settings.activeListening.prompts.promptLabelPlaceholder",
                )}
                variant="compact"
              />
            </div>

            <div className="space-y-2 flex flex-col">
              <label className="text-sm font-semibold">
                {t("settings.activeListening.prompts.promptTemplate")}
              </label>
              <Textarea
                value={draftTemplate}
                onChange={(e) => setDraftTemplate(e.target.value)}
                placeholder={t(
                  "settings.activeListening.prompts.promptTemplatePlaceholder",
                )}
                rows={6}
              />
              <p
                className="text-xs text-mid-gray/70"
                dangerouslySetInnerHTML={{
                  __html: t("settings.activeListening.prompts.promptTip"),
                }}
              />
            </div>

            <div className="flex gap-2 pt-2">
              <Button
                onClick={handleCreatePrompt}
                variant="primary"
                size="md"
                disabled={!draftName.trim() || !draftTemplate.trim()}
              >
                {t("settings.activeListening.prompts.createPrompt")}
              </Button>
              <Button
                onClick={handleCancelCreate}
                variant="secondary"
                size="md"
              >
                {t("settings.activeListening.prompts.cancel")}
              </Button>
            </div>
          </div>
        )}
      </div>
    </SettingContainer>
  );
};

export const ActiveListeningSettings: React.FC = () => {
  const { t } = useTranslation();
  const { getSetting, refreshSettings } = useSettings();

  const activeListening = getSetting("active_listening");
  const enabled = activeListening?.enabled ?? false;

  const handleEnableChange = async (value: boolean) => {
    await commands.changeActiveListeningEnabledSetting(value);
    await refreshSettings();
  };

  return (
    <div className="max-w-3xl w-full mx-auto space-y-6">
      <SettingsGroup title={t("settings.activeListening.general.title")}>
        <ToggleSwitch
          label={t("settings.activeListening.enable.title")}
          description={t("settings.activeListening.enable.description")}
          checked={enabled}
          onChange={handleEnableChange}
          grouped={true}
        />
        {enabled && (
          <HandyShortcut shortcutId="active_listening" grouped={true} />
        )}
      </SettingsGroup>

      {enabled && (
        <>
          <SettingsGroup title={t("settings.activeListening.ollama.title")}>
            <OllamaSettingsComponent />
          </SettingsGroup>

          <SettingsGroup title={t("settings.activeListening.segments.title")}>
            <SegmentSettingsComponent />
          </SettingsGroup>

          <AudioSourceSettings />

          <SettingsGroup title={t("settings.activeListening.prompts.title")}>
            <PromptsEditorComponent />
          </SettingsGroup>

          <SessionViewer />
        </>
      )}

      {!enabled && (
        <DisabledNotice>
          {t("settings.activeListening.disabledNotice")}
        </DisabledNotice>
      )}
    </div>
  );
};
