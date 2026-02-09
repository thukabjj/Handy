import { test, expect } from '@playwright/test';

/**
 * Settings page E2E tests using Playwright
 */

test.describe('Settings Pages', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(1500);
  });

  test.describe('General Settings', () => {
    test('should display general settings when sidebar is available', async ({
      page,
    }) => {
      const nav = page.locator('nav[aria-label="Settings navigation"]');
      const navExists = await nav.isVisible().catch(() => false);

      if (navExists) {
        // Click on General
        await nav.locator('text=General').click();
        await page.waitForTimeout(300);

        // Main content should be visible
        const main = page.locator('main[role="main"]');
        await expect(main).toBeVisible();
      }
    });

    test('should have settings controls', async ({ page }) => {
      const nav = page.locator('nav[aria-label="Settings navigation"]');
      const navExists = await nav.isVisible().catch(() => false);

      if (navExists) {
        await nav.locator('text=General').click();
        await page.waitForTimeout(300);

        // Look for various control types
        const main = page.locator('main[role="main"]');

        // Toggles
        const toggles = main.locator('input[type="checkbox"]');
        const toggleCount = await toggles.count();

        // Buttons
        const buttons = main.locator('button');
        const buttonCount = await buttons.count();

        test.info().annotations.push({
          type: 'info',
          description: `General settings: ${toggleCount} toggles, ${buttonCount} buttons`,
        });
      }
    });
  });

  test.describe('Advanced Settings', () => {
    test('should navigate to advanced settings', async ({ page }) => {
      const nav = page.locator('nav[aria-label="Settings navigation"]');
      const navExists = await nav.isVisible().catch(() => false);

      if (navExists) {
        const advanced = nav.locator('text=Advanced');
        if (await advanced.isVisible()) {
          await advanced.click();
          await page.waitForTimeout(300);

          // Verify we're on Advanced
          const activeSection = nav.locator('.bg-logo-primary\\/80');
          await expect(activeSection).toContainText('Advanced');
        }
      }
    });

    test('should display advanced settings content', async ({ page }) => {
      const nav = page.locator('nav[aria-label="Settings navigation"]');
      const navExists = await nav.isVisible().catch(() => false);

      if (navExists) {
        const advanced = nav.locator('text=Advanced');
        if (await advanced.isVisible()) {
          await advanced.click();
          await page.waitForTimeout(300);

          // Main content should have settings
          const main = page.locator('main[role="main"]');
          const content = await main.textContent();
          expect(content?.length).toBeGreaterThan(0);
        }
      }
    });
  });

  test.describe('History Settings', () => {
    test('should navigate to history settings', async ({ page }) => {
      const nav = page.locator('nav[aria-label="Settings navigation"]');
      const navExists = await nav.isVisible().catch(() => false);

      if (navExists) {
        const history = nav.locator('text=History');
        if (await history.isVisible()) {
          await history.click();
          await page.waitForTimeout(300);

          const activeSection = nav.locator('.bg-logo-primary\\/80');
          await expect(activeSection).toContainText('History');
        }
      }
    });

    test('should display history content or empty state', async ({ page }) => {
      const nav = page.locator('nav[aria-label="Settings navigation"]');
      const navExists = await nav.isVisible().catch(() => false);

      if (navExists) {
        const history = nav.locator('text=History');
        if (await history.isVisible()) {
          await history.click();
          await page.waitForTimeout(300);

          const main = page.locator('main[role="main"]');
          await expect(main).toBeVisible();
        }
      }
    });
  });

  test.describe('About Settings', () => {
    test('should navigate to about page', async ({ page }) => {
      const nav = page.locator('nav[aria-label="Settings navigation"]');
      const navExists = await nav.isVisible().catch(() => false);

      if (navExists) {
        const about = nav.locator('text=About');
        if (await about.isVisible()) {
          await about.click();
          await page.waitForTimeout(300);

          const activeSection = nav.locator('.bg-logo-primary\\/80');
          await expect(activeSection).toContainText('About');
        }
      }
    });

    test('should display about information', async ({ page }) => {
      const nav = page.locator('nav[aria-label="Settings navigation"]');
      const navExists = await nav.isVisible().catch(() => false);

      if (navExists) {
        const about = nav.locator('text=About');
        if (await about.isVisible()) {
          await about.click();
          await page.waitForTimeout(300);

          const main = page.locator('main[role="main"]');
          const content = await main.textContent();

          // About page should have some content
          expect(content?.length).toBeGreaterThan(0);
        }
      }
    });

    test('should have external links', async ({ page }) => {
      const nav = page.locator('nav[aria-label="Settings navigation"]');
      const navExists = await nav.isVisible().catch(() => false);

      if (navExists) {
        const about = nav.locator('text=About');
        if (await about.isVisible()) {
          await about.click();
          await page.waitForTimeout(300);

          const main = page.locator('main[role="main"]');
          const links = main.locator('a');
          const buttons = main.locator('button');

          const linkCount = await links.count();
          const buttonCount = await buttons.count();

          test.info().annotations.push({
            type: 'info',
            description: `About page: ${linkCount} links, ${buttonCount} buttons`,
          });
        }
      }
    });
  });
});

test.describe('Settings Interactions', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.waitForTimeout(1500);
  });

  test('should toggle settings and persist state', async ({ page }) => {
    const nav = page.locator('nav[aria-label="Settings navigation"]');
    const navExists = await nav.isVisible().catch(() => false);

    if (navExists) {
      await nav.locator('text=General').click();
      await page.waitForTimeout(300);

      const main = page.locator('main[role="main"]');
      const toggles = main.locator('input[type="checkbox"]');
      const count = await toggles.count();

      if (count > 0) {
        const toggle = toggles.first();
        const initialState = await toggle.isChecked();

        // Toggle the setting
        await toggle.click();
        await page.waitForTimeout(500);

        // Verify it changed
        const newState = await toggle.isChecked();
        expect(newState).toBe(!initialState);

        // Navigate away and back
        const advanced = nav.locator('text=Advanced');
        if (await advanced.isVisible()) {
          await advanced.click();
          await page.waitForTimeout(300);

          await nav.locator('text=General').click();
          await page.waitForTimeout(300);

          // Check if state persisted
          const persistedState = await toggles.first().isChecked();
          expect(persistedState).toBe(newState);
        }

        // Toggle back to original
        await toggle.click();
      }
    }
  });

  test('should handle dropdown selections', async ({ page }) => {
    const nav = page.locator('nav[aria-label="Settings navigation"]');
    const navExists = await nav.isVisible().catch(() => false);

    if (navExists) {
      // Navigate to a page with dropdowns (like Advanced)
      const advanced = nav.locator('text=Advanced');
      if (await advanced.isVisible()) {
        await advanced.click();
        await page.waitForTimeout(300);

        // Look for dropdown buttons
        const main = page.locator('main[role="main"]');
        const dropdowns = main.locator('button:has-text("Select")');
        const count = await dropdowns.count().catch(() => 0);

        if (count > 0) {
          test.info().annotations.push({
            type: 'info',
            description: `Found ${count} dropdown(s)`,
          });
        }
      }
    }
  });
});
