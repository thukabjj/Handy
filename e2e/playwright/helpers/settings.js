import { expect } from "@playwright/test";

export async function gotoApp(page) {
  await page.goto("/");
  await page.waitForTimeout(1200);
}

export async function openSidebarSection(page, label) {
  const nav = page.locator('nav[aria-label="Settings navigation"]');
  await expect(nav).toBeVisible();
  await nav.getByText(label, { exact: true }).click();
  await page.waitForTimeout(250);
  return nav;
}

export async function openSettings(page) {
  await openSidebarSection(page, "Settings");
  const main = page.locator('main[role="main"]');
  await expect(main).toBeVisible();
  return main;
}

export async function ensureUnifiedSectionExpanded(page, sectionId) {
  const toggle = page.getByTestId(`unified-section-${sectionId}-toggle`);
  await expect(toggle).toBeVisible();
  if ((await toggle.getAttribute("aria-expanded")) !== "true") {
    await toggle.click();
    await page.waitForTimeout(200);
  }
  return page.locator('main[role="main"]');
}

export async function openFeaturePanel(page, featureId) {
  await openSettings(page);
  await ensureUnifiedSectionExpanded(page, "features");
  const configureButton = page.getByTestId(`feature-configure-${featureId}`);
  await expect(configureButton).toBeVisible();
  await configureButton.click();
  await page.waitForTimeout(250);
  return page.locator('main[role="main"]');
}

export async function openSearch(page) {
  const shortcut = process.platform === "darwin" ? "Meta+KeyK" : "Control+KeyK";
  await page.keyboard.press(shortcut);
  const searchInput = page.locator('input[placeholder="Search settings..."]');
  await expect(searchInput).toBeVisible();
  return searchInput;
}

export function getSettingRowByHeading(scope, headingText) {
  return scope
    .getByRole("heading", { name: headingText, exact: true })
    .first()
    .locator('xpath=../../../div[contains(@class,"relative")][1]');
}

export async function toggleSettingByHeading(scope, headingText) {
  const control = getSettingRowByHeading(scope, headingText);
  await expect(control).toBeVisible();
  await control.locator("label").first().click();
  return control;
}

export async function selectDropdownByHeading(scope, headingText, optionText) {
  const control = getSettingRowByHeading(scope, headingText);
  await expect(control).toBeVisible();
  await control.getByRole("combobox").click();
  await scope
    .page()
    .locator('[role="option"]', { hasText: optionText })
    .first()
    .click();
  await scope.page().waitForTimeout(250);
}
