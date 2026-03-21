import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import { AskAiSettings } from "./AskAiSettings";

const mockUseSettings = vi.fn();

vi.mock("../../../hooks/useSettings", () => ({
  useSettings: () => mockUseSettings(),
}));

vi.mock("./ConversationHistory", () => ({
  ConversationHistory: () => <div data-testid="conversation-history" />,
}));

vi.mock("../ShortcutInput", () => ({
  ShortcutInput: () => <div data-testid="shortcut-input" />,
}));

describe("AskAiSettings centralized provider UI", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("shows centralized notice and model override when enabled", () => {
    mockUseSettings.mockReturnValue({
      settings: null,
      getSetting: (key: string) => {
        if (key === "ask_ai")
          return { enabled: true, system_prompt: "", llm_model: "gpt-4o-mini" };
        if (key === "active_listening") return { llm_provider: "open_ai" };
        if (key === "bindings") return { ask_ai: { id: "ask_ai" } };
        return undefined;
      },
      refreshSettings: vi.fn(),
    });

    render(<AskAiSettings />);

    expect(
      screen.getByText("settings.askAi.llm.sharedConfigNotice"),
    ).toBeInTheDocument();
    expect(
      screen.getByText("settings.askAi.llm.modelOverride.title"),
    ).toBeInTheDocument();
  });

  it("uses ollama override field when centralized provider is ollama", () => {
    mockUseSettings.mockReturnValue({
      settings: null,
      getSetting: (key: string) => {
        if (key === "ask_ai")
          return { enabled: true, system_prompt: "", ollama_model: "llama3.2" };
        if (key === "active_listening") return { llm_provider: "ollama" };
        if (key === "bindings") return { ask_ai: { id: "ask_ai" } };
        return undefined;
      },
      refreshSettings: vi.fn(),
    });

    render(<AskAiSettings />);

    expect(screen.getByDisplayValue("llama3.2")).toBeInTheDocument();
  });

  it("hides shortcut input when ask_ai binding is missing", () => {
    mockUseSettings.mockReturnValue({
      settings: null,
      getSetting: (key: string) => {
        if (key === "ask_ai") return { enabled: true, system_prompt: "" };
        if (key === "active_listening") return { llm_provider: "open_router" };
        if (key === "bindings") return {};
        return undefined;
      },
      refreshSettings: vi.fn(),
    });

    render(<AskAiSettings />);

    expect(screen.queryByTestId("shortcut-input")).not.toBeInTheDocument();
  });
});
