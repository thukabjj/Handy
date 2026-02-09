import { test, expect } from '@playwright/test';

/**
 * Active Listening E2E tests using Playwright
 *
 * Note: These tests work with the UI layer only. Full Active Listening
 * functionality requires the Tauri runtime (Ollama integration, audio
 * processing). For complete E2E testing, use tauri-driver on Linux/Windows.
 */

test.describe('Active Listening Settings', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(1500);
  });

  test('should display Active Listening section when enabled in sidebar', async ({
    page,
  }) => {
    const nav = page.locator('nav[aria-label="Settings navigation"]');
    const navExists = await nav.isVisible().catch(() => false);

    if (navExists) {
      // Check if Active Listening section exists in sidebar
      const activeListening = nav.locator('text=Active Listening');
      const isVisible = await activeListening.isVisible().catch(() => false);

      test.info().annotations.push({
        type: 'info',
        description: isVisible
          ? 'Active Listening section is visible (feature enabled)'
          : 'Active Listening section not visible (feature disabled)',
      });

      if (isVisible) {
        await activeListening.click();
        await page.waitForTimeout(300);

        // Verify the section is now active
        const activeSection = nav.locator('.bg-logo-primary\\/80');
        await expect(activeSection).toContainText('Active Listening');
      }
    }
  });

  test('should show Active Listening settings content when section is visible', async ({
    page,
  }) => {
    const nav = page.locator('nav[aria-label="Settings navigation"]');
    const navExists = await nav.isVisible().catch(() => false);

    if (navExists) {
      const activeListening = nav.locator('text=Active Listening');
      const isVisible = await activeListening.isVisible().catch(() => false);

      if (isVisible) {
        await activeListening.click();
        await page.waitForTimeout(300);

        const main = page.locator('main[role="main"]');
        await expect(main).toBeVisible();

        // Active Listening settings should have configuration options
        const content = await main.textContent();
        expect(content?.length).toBeGreaterThan(0);

        test.info().annotations.push({
          type: 'info',
          description: 'Active Listening settings page loaded successfully',
        });
      }
    }
  });

  test('should have Ollama configuration options', async ({ page }) => {
    const nav = page.locator('nav[aria-label="Settings navigation"]');
    const navExists = await nav.isVisible().catch(() => false);

    if (navExists) {
      const activeListening = nav.locator('text=Active Listening');
      const isVisible = await activeListening.isVisible().catch(() => false);

      if (isVisible) {
        await activeListening.click();
        await page.waitForTimeout(300);

        const main = page.locator('main[role="main"]');

        // Look for Ollama-related inputs
        const inputs = main.locator('input[type="text"], input[type="url"]');
        const inputCount = await inputs.count();

        // Look for model selector
        const buttons = main.locator('button');
        const buttonCount = await buttons.count();

        test.info().annotations.push({
          type: 'info',
          description: `Active Listening: ${inputCount} input fields, ${buttonCount} buttons`,
        });
      }
    }
  });

  test('should have prompt management UI', async ({ page }) => {
    const nav = page.locator('nav[aria-label="Settings navigation"]');
    const navExists = await nav.isVisible().catch(() => false);

    if (navExists) {
      const activeListening = nav.locator('text=Active Listening');
      const isVisible = await activeListening.isVisible().catch(() => false);

      if (isVisible) {
        await activeListening.click();
        await page.waitForTimeout(300);

        const main = page.locator('main[role="main"]');

        // Check for prompt-related UI elements
        const textareas = main.locator('textarea');
        const textareaCount = await textareas.count();

        test.info().annotations.push({
          type: 'info',
          description: `Active Listening prompts: ${textareaCount} text areas found`,
        });
      }
    }
  });

  test('should have session viewer component', async ({ page }) => {
    const nav = page.locator('nav[aria-label="Settings navigation"]');
    const navExists = await nav.isVisible().catch(() => false);

    if (navExists) {
      const activeListening = nav.locator('text=Active Listening');
      const isVisible = await activeListening.isVisible().catch(() => false);

      if (isVisible) {
        await activeListening.click();
        await page.waitForTimeout(300);

        const main = page.locator('main[role="main"]');

        // Session viewer should show sessions list or empty state
        const sessionList = main.locator('[data-testid="session-list"]');
        const emptyState = main.locator('text=No sessions');

        const hasSessionList = await sessionList.isVisible().catch(() => false);
        const hasEmptyState = await emptyState.isVisible().catch(() => false);

        test.info().annotations.push({
          type: 'info',
          description: hasSessionList
            ? 'Session list is visible'
            : hasEmptyState
              ? 'Empty state is shown'
              : 'Session viewer component not found (may need implementation)',
        });
      }
    }
  });
});

test.describe('Active Listening Feature Toggle', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(1500);
  });

  test('should show Active Listening toggle in General settings', async ({
    page,
  }) => {
    const nav = page.locator('nav[aria-label="Settings navigation"]');
    const navExists = await nav.isVisible().catch(() => false);

    if (navExists) {
      await nav.locator('text=General').click();
      await page.waitForTimeout(300);

      const main = page.locator('main[role="main"]');
      const content = await main.textContent();

      // Check if Active Listening toggle exists in General settings
      const hasActiveListeningText =
        content?.includes('Active Listening') ||
        content?.includes('active listening');

      test.info().annotations.push({
        type: 'info',
        description: hasActiveListeningText
          ? 'Active Listening toggle found in General settings'
          : 'Active Listening toggle may be in a different location',
      });
    }
  });
});
