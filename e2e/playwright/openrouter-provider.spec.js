import { test, expect } from "@playwright/test";
import { gotoApp, openFeaturePanel } from "./helpers/settings";

async function selectComboboxOption(page, container, optionText) {
  await container.getByRole("combobox").click();
  await page
    .locator('[role="option"]', { hasText: optionText })
    .first()
    .click();
  await page.waitForTimeout(250);
}

test.describe("OpenRouter Provider Flow", () => {
  test.beforeEach(async ({ page }) => {
    await gotoApp(page);
  });

  test("should configure shared AI settings in Cloud mode with OpenRouter", async ({
    page,
  }) => {
    const main = await openFeaturePanel(page, "ai-config");

    await selectComboboxOption(
      page,
      page.getByTestId("shared-ai-mode"),
      "Cloud",
    );
    await selectComboboxOption(
      page,
      page.getByTestId("shared-ai-cloud-provider"),
      "OpenRouter",
    );

    await expect(main.getByText("API Key")).toBeVisible();
    await expect(
      main.getByRole("heading", { name: "Default Model", exact: true }),
    ).toBeVisible();
    await expect(main.getByText("Base URL")).toBeHidden();
  });

  test("should show the base URL field only for the custom cloud provider", async ({
    page,
  }) => {
    const main = await openFeaturePanel(page, "ai-config");

    await selectComboboxOption(
      page,
      page.getByTestId("shared-ai-mode"),
      "Cloud",
    );
    await selectComboboxOption(
      page,
      page.getByTestId("shared-ai-cloud-provider"),
      "OpenRouter",
    );
    await expect(main.getByText("Base URL")).toBeHidden();

    await selectComboboxOption(
      page,
      page.getByTestId("shared-ai-cloud-provider"),
      "Custom endpoint (OpenAI-compatible)",
    );
    await expect(main.getByText("Base URL")).toBeVisible();
  });

  test("should expose Ask AI shared-provider messaging after central AI configuration", async ({
    page,
  }) => {
    const main = await openFeaturePanel(page, "ai-config");

    await selectComboboxOption(
      page,
      page.getByTestId("shared-ai-mode"),
      "Cloud",
    );
    await selectComboboxOption(
      page,
      page.getByTestId("shared-ai-cloud-provider"),
      "OpenRouter",
    );

    const askAiMain = await openFeaturePanel(page, "ask-ai");
    await page
      .locator('main[role="main"]')
      .getByRole("heading", { name: "Enable Ask AI", exact: true })
      .first()
      .locator('xpath=../../../div[contains(@class,"relative")][1]//label[1]')
      .click();
    await page.waitForTimeout(250);

    await expect(
      askAiMain.getByText("AI provider is configured centrally"),
    ).toBeVisible();
    await expect(askAiMain.getByText("Model Override")).toBeVisible();
    await expect(main).toBeVisible();
  });

  test("should allow editing API keys for non-OpenRouter cloud providers", async ({
    page,
  }) => {
    const main = await openFeaturePanel(page, "ai-config");

    await selectComboboxOption(
      page,
      page.getByTestId("shared-ai-mode"),
      "Cloud",
    );
    await selectComboboxOption(
      page,
      page.getByTestId("shared-ai-cloud-provider"),
      "OpenAI",
    );

    const apiKeyInput = page
      .getByTestId("shared-ai-api-key")
      .locator("input")
      .first();
    await expect(apiKeyInput).toBeVisible();
    await apiKeyInput.fill("sk-test-openai-key");
    await apiKeyInput.blur();
    await expect(apiKeyInput).toHaveValue("sk-test-openai-key");
  });
});
