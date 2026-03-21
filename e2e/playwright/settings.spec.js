import { test, expect } from "@playwright/test";
import {
  ensureUnifiedSectionExpanded,
  gotoApp,
  openSettings,
  openSidebarSection,
} from "./helpers/settings";

test.describe("Unified Settings", () => {
  test.beforeEach(async ({ page }) => {
    await gotoApp(page);
  });

  test("should show the unified settings sections", async ({ page }) => {
    await openSettings(page);

    await expect(
      page.getByTestId("unified-section-audio-toggle"),
    ).toBeVisible();
    await expect(
      page.getByTestId("unified-section-output-toggle"),
    ).toBeVisible();
    await expect(
      page.getByTestId("unified-section-transcription-toggle"),
    ).toBeVisible();
    await expect(page.getByTestId("unified-section-app-toggle")).toBeVisible();
    await expect(
      page.getByTestId("unified-section-features-toggle"),
    ).toBeVisible();
  });

  test("should expose the feature cards inside advanced features", async ({
    page,
  }) => {
    await openSettings(page);
    await ensureUnifiedSectionExpanded(page, "features");

    await expect(page.getByTestId("feature-card-ai-config")).toBeVisible();
    await expect(page.getByTestId("feature-card-knowledge-base")).toBeVisible();
    await expect(
      page.getByTestId("feature-card-active-listening"),
    ).toBeVisible();
    await expect(page.getByTestId("feature-card-ask-ai")).toBeVisible();
    await expect(
      page.getByTestId("feature-card-batch-processing"),
    ).toBeVisible();
    await expect(page.getByTestId("feature-card-suggestions")).toBeVisible();
    await expect(page.getByTestId("feature-card-diagnostics")).toBeVisible();
  });

  test("should persist a feature checkbox after navigation", async ({
    page,
  }) => {
    await openSettings(page);
    await ensureUnifiedSectionExpanded(page, "features");

    const card = page.getByTestId("feature-card-knowledge-base");
    const label = card.locator("label").first();
    const checkbox = card.locator('input[type="checkbox"]').first();
    const initialState = await checkbox.isChecked();

    await label.click();
    await page.waitForTimeout(300);
    await expect(checkbox).toHaveJSProperty("checked", !initialState);

    await openSidebarSection(page, "History");
    await openSettings(page);
    await ensureUnifiedSectionExpanded(page, "features");
    await expect(
      page
        .getByTestId("feature-card-knowledge-base")
        .locator('input[type="checkbox"]')
        .first(),
    ).toHaveJSProperty("checked", !initialState);

    await page
      .getByTestId("feature-card-knowledge-base")
      .locator("label")
      .first()
      .click();
    await page.waitForTimeout(200);
  });

  test("should expose interactive dropdowns in the audio section", async ({
    page,
  }) => {
    await openSettings(page);
    await ensureUnifiedSectionExpanded(page, "audio");

    const dropdowns = page.locator('main[role="main"] button[role="combobox"]');
    await expect(dropdowns.first()).toBeVisible();
    expect(await dropdowns.count()).toBeGreaterThan(0);
  });
});
