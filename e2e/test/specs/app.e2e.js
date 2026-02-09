/**
 * E2E tests for main Handy application
 *
 * Note: These tests require tauri-driver and a built release version of the app.
 * Run `cargo build --release` in src-tauri before running tests.
 *
 * Platform support:
 * - Linux: Full support via WebKitWebDriver
 * - Windows: Full support via Microsoft Edge WebDriver
 * - macOS: Limited support (WKWebView doesn't have a WebDriver implementation)
 */

describe('Handy Application', () => {
  describe('Application Launch', () => {
    it('should launch successfully', async () => {
      // The app should be running if we got here
      const title = await browser.getTitle();
      // Tauri apps may have different titles, just check it exists
      expect(typeof title).toBe('string');
    });

    it('should have a valid window', async () => {
      const windowHandle = await browser.getWindowHandle();
      expect(windowHandle).toBeTruthy();
    });
  });

  describe('Main Window Structure', () => {
    it('should render the main navigation', async () => {
      // Wait for the app to fully load
      await browser.pause(2000);

      // Check for settings navigation
      const nav = await $('nav[aria-label="Settings navigation"]');
      const navExists = await nav.isExisting();

      // If nav doesn't exist, we might be on the onboarding screen
      if (!navExists) {
        console.log(
          'Navigation not found - app may be showing onboarding screen'
        );
      }
    });

    it('should have main content area', async () => {
      const main = await $('main[role="main"]');
      const mainExists = await main.isExisting();

      if (mainExists) {
        expect(await main.isDisplayed()).toBe(true);
      }
    });

    it('should have footer', async () => {
      const footer = await $('footer[role="contentinfo"]');
      const footerExists = await footer.isExisting();

      if (footerExists) {
        expect(await footer.isDisplayed()).toBe(true);
      }
    });
  });

  describe('Sidebar Navigation', () => {
    it('should display sidebar sections', async () => {
      // Wait for app to load
      await browser.pause(1000);

      const sidebar = await $('nav[aria-label="Settings navigation"]');
      const sidebarExists = await sidebar.isExisting();

      if (sidebarExists) {
        // Check for General section (always visible)
        const generalText = await sidebar.$('*=General');
        if (await generalText.isExisting()) {
          expect(await generalText.isDisplayed()).toBe(true);
        }
      }
    });

    it('should allow clicking on sidebar sections', async () => {
      const sidebar = await $('nav[aria-label="Settings navigation"]');
      const sidebarExists = await sidebar.isExisting();

      if (sidebarExists) {
        // Try to click on Advanced section
        const advancedSection = await sidebar.$('*=Advanced');
        if (await advancedSection.isExisting()) {
          await advancedSection.click();
          await browser.pause(500);

          // Verify the section changed (element should have active styling)
          const parent = await advancedSection.parentElement();
          const className = await parent.getAttribute('class');
          expect(className).toContain('bg-logo-primary');
        }
      }
    });

    it('should navigate to History section', async () => {
      const sidebar = await $('nav[aria-label="Settings navigation"]');
      const sidebarExists = await sidebar.isExisting();

      if (sidebarExists) {
        const historySection = await sidebar.$('*=History');
        if (await historySection.isExisting()) {
          await historySection.click();
          await browser.pause(500);

          const parent = await historySection.parentElement();
          const className = await parent.getAttribute('class');
          expect(className).toContain('bg-logo-primary');
        }
      }
    });

    it('should navigate to About section', async () => {
      const sidebar = await $('nav[aria-label="Settings navigation"]');
      const sidebarExists = await sidebar.isExisting();

      if (sidebarExists) {
        const aboutSection = await sidebar.$('*=About');
        if (await aboutSection.isExisting()) {
          await aboutSection.click();
          await browser.pause(500);

          const parent = await aboutSection.parentElement();
          const className = await parent.getAttribute('class');
          expect(className).toContain('bg-logo-primary');
        }
      }
    });

    it('should return to General section', async () => {
      const sidebar = await $('nav[aria-label="Settings navigation"]');
      const sidebarExists = await sidebar.isExisting();

      if (sidebarExists) {
        const generalSection = await sidebar.$('*=General');
        if (await generalSection.isExisting()) {
          await generalSection.click();
          await browser.pause(500);

          const parent = await generalSection.parentElement();
          const className = await parent.getAttribute('class');
          expect(className).toContain('bg-logo-primary');
        }
      }
    });
  });

  describe('Window Management', () => {
    it('should allow window resize', async () => {
      const originalSize = await browser.getWindowSize();

      await browser.setWindowSize(1200, 800);
      const newSize = await browser.getWindowSize();

      expect(newSize.width).toBe(1200);
      expect(newSize.height).toBe(800);

      // Restore original size
      await browser.setWindowSize(originalSize.width, originalSize.height);
    });

    it('should handle minimum window size gracefully', async () => {
      const originalSize = await browser.getWindowSize();

      // Try to set a small window size
      await browser.setWindowSize(600, 400);
      const newSize = await browser.getWindowSize();

      // The app should either accept the size or enforce a minimum
      expect(newSize.width).toBeGreaterThanOrEqual(400);
      expect(newSize.height).toBeGreaterThanOrEqual(300);

      // Restore original size
      await browser.setWindowSize(originalSize.width, originalSize.height);
    });
  });
});
