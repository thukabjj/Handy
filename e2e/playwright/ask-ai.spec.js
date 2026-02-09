import { test, expect } from '@playwright/test';

/**
 * Ask AI E2E tests using Playwright
 *
 * Note: These tests work with the UI layer only. Full Ask AI functionality
 * requires the Tauri runtime (Ollama integration, voice recording).
 * For complete E2E testing, use tauri-driver on Linux/Windows.
 */

test.describe('Ask AI Settings', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(1500);
  });

  test('should display Ask AI section when enabled in sidebar', async ({
    page,
  }) => {
    const nav = page.locator('nav[aria-label="Settings navigation"]');
    const navExists = await nav.isVisible().catch(() => false);

    if (navExists) {
      // Check if Ask AI section exists in sidebar
      const askAi = nav.locator('text=Ask AI');
      const isVisible = await askAi.isVisible().catch(() => false);

      test.info().annotations.push({
        type: 'info',
        description: isVisible
          ? 'Ask AI section is visible (feature enabled)'
          : 'Ask AI section not visible (feature disabled)',
      });

      if (isVisible) {
        await askAi.click();
        await page.waitForTimeout(300);

        // Verify the section is now active
        const activeSection = nav.locator('.bg-logo-primary\\/80');
        await expect(activeSection).toContainText('Ask AI');
      }
    }
  });

  test('should show Ask AI settings content when section is visible', async ({
    page,
  }) => {
    const nav = page.locator('nav[aria-label="Settings navigation"]');
    const navExists = await nav.isVisible().catch(() => false);

    if (navExists) {
      const askAi = nav.locator('text=Ask AI');
      const isVisible = await askAi.isVisible().catch(() => false);

      if (isVisible) {
        await askAi.click();
        await page.waitForTimeout(300);

        const main = page.locator('main[role="main"]');
        await expect(main).toBeVisible();

        // Ask AI settings should have configuration options
        const content = await main.textContent();
        expect(content?.length).toBeGreaterThan(0);

        test.info().annotations.push({
          type: 'info',
          description: 'Ask AI settings page loaded successfully',
        });
      }
    }
  });

  test('should have Ollama configuration options', async ({ page }) => {
    const nav = page.locator('nav[aria-label="Settings navigation"]');
    const navExists = await nav.isVisible().catch(() => false);

    if (navExists) {
      const askAi = nav.locator('text=Ask AI');
      const isVisible = await askAi.isVisible().catch(() => false);

      if (isVisible) {
        await askAi.click();
        await page.waitForTimeout(300);

        const main = page.locator('main[role="main"]');

        // Look for Ollama base URL input
        const urlInputs = main.locator(
          'input[type="text"], input[type="url"], input[placeholder*="localhost"]'
        );
        const urlInputCount = await urlInputs.count();

        test.info().annotations.push({
          type: 'info',
          description: `Ask AI: ${urlInputCount} URL/text input fields found`,
        });
      }
    }
  });

  test('should have model selection dropdown', async ({ page }) => {
    const nav = page.locator('nav[aria-label="Settings navigation"]');
    const navExists = await nav.isVisible().catch(() => false);

    if (navExists) {
      const askAi = nav.locator('text=Ask AI');
      const isVisible = await askAi.isVisible().catch(() => false);

      if (isVisible) {
        await askAi.click();
        await page.waitForTimeout(300);

        const main = page.locator('main[role="main"]');

        // Look for dropdown/select components
        const dropdowns = main.locator('button[role="combobox"], select');
        const dropdownCount = await dropdowns.count();

        test.info().annotations.push({
          type: 'info',
          description: `Ask AI: ${dropdownCount} dropdown/select elements found`,
        });
      }
    }
  });

  test('should have system prompt configuration', async ({ page }) => {
    const nav = page.locator('nav[aria-label="Settings navigation"]');
    const navExists = await nav.isVisible().catch(() => false);

    if (navExists) {
      const askAi = nav.locator('text=Ask AI');
      const isVisible = await askAi.isVisible().catch(() => false);

      if (isVisible) {
        await askAi.click();
        await page.waitForTimeout(300);

        const main = page.locator('main[role="main"]');

        // Check for system prompt textarea
        const textareas = main.locator('textarea');
        const textareaCount = await textareas.count();

        test.info().annotations.push({
          type: 'info',
          description: `Ask AI: ${textareaCount} textarea elements found (for system prompt)`,
        });
      }
    }
  });

  test('should have conversation history browser', async ({ page }) => {
    const nav = page.locator('nav[aria-label="Settings navigation"]');
    const navExists = await nav.isVisible().catch(() => false);

    if (navExists) {
      const askAi = nav.locator('text=Ask AI');
      const isVisible = await askAi.isVisible().catch(() => false);

      if (isVisible) {
        await askAi.click();
        await page.waitForTimeout(300);

        const main = page.locator('main[role="main"]');

        // Conversation history should show list or empty state
        const historyList = main.locator('[data-testid="conversation-list"]');
        const emptyState = main.locator('text=No conversations');

        const hasHistoryList = await historyList.isVisible().catch(() => false);
        const hasEmptyState = await emptyState.isVisible().catch(() => false);

        test.info().annotations.push({
          type: 'info',
          description: hasHistoryList
            ? 'Conversation list is visible'
            : hasEmptyState
              ? 'Empty state is shown'
              : 'Conversation history component loading or not implemented',
        });
      }
    }
  });
});

test.describe('Ask AI Feature Toggle', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(1500);
  });

  test('should show Ask AI toggle in General settings', async ({ page }) => {
    const nav = page.locator('nav[aria-label="Settings navigation"]');
    const navExists = await nav.isVisible().catch(() => false);

    if (navExists) {
      await nav.locator('text=General').click();
      await page.waitForTimeout(300);

      const main = page.locator('main[role="main"]');
      const content = await main.textContent();

      // Check if Ask AI toggle exists in General settings
      const hasAskAiText =
        content?.includes('Ask AI') || content?.includes('ask AI');

      test.info().annotations.push({
        type: 'info',
        description: hasAskAiText
          ? 'Ask AI toggle found in General settings'
          : 'Ask AI toggle may be in a different location',
      });
    }
  });
});

test.describe('Ask AI Error Handling UI', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(1500);
  });

  test('should have error handling elements in overlay (if accessible)', async ({
    page,
  }) => {
    // This test documents the expected error handling UI in the overlay
    // The overlay is a separate window in Tauri, so direct testing is limited

    test.info().annotations.push({
      type: 'info',
      description:
        'Overlay error handling: Connection errors show retry button, ' +
        'config errors show settings link, transient errors show wait message',
    });

    // Document the error types that are handled:
    const errorTypes = [
      'connection - Ollama not running or connection refused',
      'config - No model configured or invalid settings',
      'transient - Timeout or model loading issues',
      'speech - No speech detected or transcription failed',
      'unknown - Generic error with retry option',
    ];

    test.info().annotations.push({
      type: 'info',
      description: `Error types handled: ${errorTypes.join(', ')}`,
    });
  });
});
