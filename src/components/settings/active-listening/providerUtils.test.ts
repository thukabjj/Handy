import { describe, it, expect } from "vitest";
import {
  CLOUD_PROVIDER_PRESETS,
  detectCloudProvider,
  prioritizeOpenRouterModels,
  resolveCloudFetchConfig,
} from "./providerUtils";

describe("providerUtils", () => {
  it("detects direct cloud provider variants", () => {
    expect(detectCloudProvider("open_router", "")).toBe("open_router");
    expect(detectCloudProvider("open_ai", "")).toBe("open_ai");
    expect(detectCloudProvider("groq", "")).toBe("groq");
    expect(detectCloudProvider("together", "")).toBe("together");
    expect(detectCloudProvider("fireworks", "")).toBe("fireworks");
  });

  it("detects custom mapped provider by base URL", () => {
    expect(detectCloudProvider("custom", CLOUD_PROVIDER_PRESETS.open_ai)).toBe(
      "open_ai",
    );
    expect(detectCloudProvider("custom", CLOUD_PROVIDER_PRESETS.groq)).toBe(
      "groq",
    );
  });

  it("keeps custom provider when URL is unknown", () => {
    expect(detectCloudProvider("custom", "https://example.com/v1")).toBe(
      "custom",
    );
  });

  it("resolves cloud fetch config", () => {
    expect(resolveCloudFetchConfig("open_router", "")).toEqual({
      provider: "open_router",
      baseUrl: null,
    });
    expect(resolveCloudFetchConfig("open_ai", "")).toEqual({
      provider: "open_ai",
      baseUrl: null,
    });
    expect(resolveCloudFetchConfig("custom", "https://example.com/v1")).toEqual(
      {
        provider: "custom",
        baseUrl: "https://example.com/v1",
      },
    );
  });

  it("prioritizes free/open-source models for OpenRouter", () => {
    const sorted = prioritizeOpenRouterModels([
      "openai/gpt-4o-mini",
      "meta-llama/llama-3.1-8b-instruct:free",
      "qwen/qwen2.5-7b-instruct",
      "google/gemini-2.0-flash-001",
    ]);

    expect(sorted[0]).toBe("meta-llama/llama-3.1-8b-instruct:free");
    expect(sorted.indexOf("qwen/qwen2.5-7b-instruct")).toBeLessThan(
      sorted.indexOf("openai/gpt-4o-mini"),
    );
  });
});
