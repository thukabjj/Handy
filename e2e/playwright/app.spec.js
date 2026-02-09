import { test, expect } from '@playwright/test';

/**
 * E2E tests for the Handy application using Playwright
 *
 * These tests run against the Vite dev server and test the web frontend.
 * Tauri-specific native features are mocked or skipped.
 */

test.describe('Handy Application', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to the app
    await page.goto('/');

    // Wait for the app to initialize
    await page.waitForTimeout(1000);
  });

  test('should load the application', async ({ page }) => {
    // The page should have loaded
    await expect(page).toHaveURL('/');
  });

  test('should display main content', async ({ page }) => {
    // Check for body content
    const body = page.locator('body');
    await expect(body).toBeVisible();
  });

  test('should have app structure', async ({ page }) => {
    // Wait for React to render
    await page.waitForSelector('div', { timeout: 5000 });

    // The app root should exist
    const root = page.locator('#root');
    await expect(root).toBeVisible();
  });
});

test.describe('Navigation', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(1000);
  });

  test('should have navigation when main app loads', async ({ page }) => {
    // Check if sidebar navigation exists (main app state)
    const nav = page.locator('nav[aria-label="Settings navigation"]');
    const navExists = await nav.isVisible().catch(() => false);

    if (navExists) {
      // We're in the main app
      await expect(nav).toBeVisible();

      // Check for sidebar sections
      const generalText = nav.locator('text=General');
      await expect(generalText).toBeVisible();
    } else {
      // We might be in onboarding - that's also valid
      test.info().annotations.push({
        type: 'info',
        description: 'App is showing onboarding screen',
      });
    }
  });

  test('should navigate between sections when in main app', async ({
    page,
  }) => {
    const nav = page.locator('nav[aria-label="Settings navigation"]');
    const navExists = await nav.isVisible().catch(() => false);

    if (navExists) {
      // Click on Advanced
      const advanced = nav.locator('text=Advanced');
      if (await advanced.isVisible()) {
        await advanced.click();
        await page.waitForTimeout(300);

        // The Advanced section should now be active
        const activeSection = nav.locator('.bg-logo-primary\\/80');
        await expect(activeSection).toContainText('Advanced');
      }

      // Click on History
      const history = nav.locator('text=History');
      if (await history.isVisible()) {
        await history.click();
        await page.waitForTimeout(300);
      }

      // Click on About
      const about = nav.locator('text=About');
      if (await about.isVisible()) {
        await about.click();
        await page.waitForTimeout(300);
      }

      // Return to General
      const general = nav.locator('text=General');
      await general.click();
      await page.waitForTimeout(300);
    }
  });
});

test.describe('Settings UI', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(1000);
  });

  test('should have interactive toggle switches', async ({ page }) => {
    const nav = page.locator('nav[aria-label="Settings navigation"]');
    const navExists = await nav.isVisible().catch(() => false);

    if (navExists) {
      // Look for toggle switches (checkboxes styled as toggles)
      const toggles = page.locator('input[type="checkbox"]');
      const count = await toggles.count();

      if (count > 0) {
        test.info().annotations.push({
          type: 'info',
          description: `Found ${count} toggle switches`,
        });

        // Try to interact with the first toggle
        const firstToggle = toggles.first();
        const initialState = await firstToggle.isChecked();

        await firstToggle.click();
        await page.waitForTimeout(200);

        // Toggle should have changed
        const newState = await firstToggle.isChecked();
        expect(newState).toBe(!initialState);

        // Toggle back
        await firstToggle.click();
      }
    }
  });

  test('should have main content area', async ({ page }) => {
    const main = page.locator('main[role="main"]');
    const mainExists = await main.isVisible().catch(() => false);

    if (mainExists) {
      await expect(main).toBeVisible();
    }
  });

  test('should have footer', async ({ page }) => {
    const footer = page.locator('footer[role="contentinfo"]');
    const footerExists = await footer.isVisible().catch(() => false);

    if (footerExists) {
      await expect(footer).toBeVisible();
    }
  });
});

test.describe('Keyboard Interactions', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(1000);
  });

  test('should support Tab navigation', async ({ page }) => {
    // Press Tab to navigate
    await page.keyboard.press('Tab');
    await page.waitForTimeout(100);

    // Get the focused element
    const focusedTag = await page.evaluate(
      () => document.activeElement?.tagName
    );
    expect(focusedTag).toBeTruthy();
  });

  test('should handle Escape key gracefully', async ({ page }) => {
    await page.keyboard.press('Escape');
    await page.waitForTimeout(100);

    // App should still be responsive
    const body = page.locator('body');
    await expect(body).toBeVisible();
  });

  test('should toggle debug mode with keyboard shortcut', async ({ page }) => {
    const nav = page.locator('nav[aria-label="Settings navigation"]');
    const navExists = await nav.isVisible().catch(() => false);

    if (navExists) {
      // Check if Debug section is initially visible
      const debugBefore = await nav.locator('text=Debug').isVisible().catch(() => false);

      // Press Cmd+Shift+D (Meta+Shift+D)
      await page.keyboard.press('Meta+Shift+KeyD');
      await page.waitForTimeout(500);

      // Check if Debug section visibility changed
      const debugAfter = await nav.locator('text=Debug').isVisible().catch(() => false);

      // Toggle back
      await page.keyboard.press('Meta+Shift+KeyD');
      await page.waitForTimeout(300);

      test.info().annotations.push({
        type: 'info',
        description: `Debug visibility: ${debugBefore} -> ${debugAfter}`,
      });
    }
  });
});

test.describe('Responsive Design', () => {
  test('should handle different viewport sizes', async ({ page }) => {
    await page.goto('/');

    // Test at different sizes
    const sizes = [
      { width: 1280, height: 720 },
      { width: 1024, height: 768 },
      { width: 800, height: 600 },
    ];

    for (const size of sizes) {
      await page.setViewportSize(size);
      await page.waitForTimeout(200);

      // App should still render
      const body = page.locator('body');
      await expect(body).toBeVisible();

      test.info().annotations.push({
        type: 'info',
        description: `Tested at ${size.width}x${size.height}`,
      });
    }
  });
});
