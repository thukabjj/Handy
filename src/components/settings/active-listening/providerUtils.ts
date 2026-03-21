import type { LlmProvider } from "@/bindings";

export type CloudProviderId =
  | "open_router"
  | "open_ai"
  | "groq"
  | "together"
  | "fireworks"
  | "custom";

export const CLOUD_PROVIDER_PRESETS: Record<
  Exclude<CloudProviderId, "open_router" | "custom">,
  string
> = {
  open_ai: "https://api.openai.com/v1",
  groq: "https://api.groq.com/openai/v1",
  together: "https://api.together.xyz/v1",
  fireworks: "https://api.fireworks.ai/inference/v1",
};

const knownOpenSourceFamilies = [
  "llama",
  "mistral",
  "mixtral",
  "qwen",
  "deepseek",
  "gemma",
  "phi",
  "openchat",
  "nous",
  "dolphin",
  "falcon",
  "yi",
  "nemotron",
];

export const normalizeUrl = (url: string) => url.trim().replace(/\/+$/, "");

export const prioritizeOpenRouterModels = (models: string[]) => {
  const isLikelyOpenSource = (id: string) => {
    const lower = id.toLowerCase();
    return knownOpenSourceFamilies.some((family) => lower.includes(family));
  };

  return [...models].sort((a, b) => {
    const aScore = Number(a.includes(":free")) + Number(isLikelyOpenSource(a));
    const bScore = Number(b.includes(":free")) + Number(isLikelyOpenSource(b));
    return bScore - aScore || a.localeCompare(b);
  });
};

export const detectCloudProvider = (
  provider: LlmProvider,
  llmBaseUrl: string,
): CloudProviderId => {
  if (
    provider === "open_router" ||
    provider === "open_ai" ||
    provider === "groq" ||
    provider === "together" ||
    provider === "fireworks"
  ) {
    return provider;
  }

  if (provider !== "custom") return "open_router";

  const normalized = normalizeUrl(llmBaseUrl);
  for (const [id, url] of Object.entries(CLOUD_PROVIDER_PRESETS)) {
    if (normalized === normalizeUrl(url)) {
      return id as Exclude<CloudProviderId, "open_router" | "custom">;
    }
  }
  return "custom";
};

export const resolveCloudFetchConfig = (
  cloudProvider: CloudProviderId,
  llmBaseUrl: string,
): { provider: LlmProvider; baseUrl: string | null } => ({
  provider:
    cloudProvider === "open_router"
      ? "open_router"
      : cloudProvider === "custom"
        ? "custom"
        : (cloudProvider as LlmProvider),
  baseUrl: cloudProvider === "custom" ? llmBaseUrl || null : null,
});
