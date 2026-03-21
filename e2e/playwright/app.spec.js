import { test, expect } from "@playwright/test";
import { gotoApp, openSearch, openSidebarSection } from "./helpers/settings";

test.describe("Handy Application", () => {
  test.beforeEach(async ({ page }) => {
    await gotoApp(page);
  });

  test("should render the main shell", async ({ page }) => {
    await expect(page.locator("#root")).toBeAttached();
    await expect(page.locator('main[role="main"]')).toBeVisible();
    await expect(page.locator('footer[role="contentinfo"]')).toBeVisible();
  });

  test("should show the current sidebar navigation", async ({ page }) => {
    const nav = page.locator('nav[aria-label="Settings navigation"]');
    await expect(nav).toBeVisible();
    await expect(nav.getByText("Transcribe", { exact: true })).toBeVisible();
    await expect(nav.getByText("History", { exact: true })).toBeVisible();
    await expect(nav.getByText("Settings", { exact: true })).toBeVisible();
    await expect(nav.getByText("About", { exact: true })).toBeVisible();
  });

  test("should navigate between sidebar sections", async ({ page }) => {
    const nav = page.locator('nav[aria-label="Settings navigation"]');

    await openSidebarSection(page, "Settings");
    await expect(nav.locator(".bg-logo-primary\\/80")).toContainText(
      "Settings",
    );

    await openSidebarSection(page, "History");
    await expect(nav.locator(".bg-logo-primary\\/80")).toContainText("History");

    await openSidebarSection(page, "About");
    await expect(nav.locator(".bg-logo-primary\\/80")).toContainText("About");

    await openSidebarSection(page, "Transcribe");
    await expect(nav.locator(".bg-logo-primary\\/80")).toContainText(
      "Transcribe",
    );
  });

  test("should deep-link search results into the correct unified settings panel", async ({
    page,
  }) => {
    const searchInput = await openSearch(page);
    await searchInput.fill("Knowledge Base");
    await page.keyboard.press("Enter");
    await page.waitForTimeout(250);

    const nav = page.locator('nav[aria-label="Settings navigation"]');
    await expect(nav.locator(".bg-logo-primary\\/80")).toContainText(
      "Settings",
    );
    await expect(
      page.getByTestId("unified-section-features-toggle"),
    ).toHaveAttribute("aria-expanded", "true");
    await expect(page.getByTestId("feature-card-knowledge-base")).toBeVisible();
    await expect(
      page.locator('main[role="main"]').getByText("Knowledge Base"),
    ).toBeVisible();
  });

  test("should toggle debug mode with the keyboard shortcut", async ({
    page,
  }) => {
    const nav = page.locator('nav[aria-label="Settings navigation"]');
    const debugBefore = await nav
      .getByText("Debug", { exact: true })
      .isVisible()
      .catch(() => false);

    await page.keyboard.press("Meta+Shift+KeyD");
    await page.waitForTimeout(400);
    const debugAfter = await nav
      .getByText("Debug", { exact: true })
      .isVisible()
      .catch(() => false);

    await page.keyboard.press("Meta+Shift+KeyD");
    await page.waitForTimeout(250);

    expect(debugAfter).toBe(!debugBefore);
  });
});
