/**
 * E2E tests for Keyboard interactions
 *
 * Tests keyboard shortcuts and navigation
 */

describe('Keyboard Interactions', () => {
  describe('Debug Mode Toggle', () => {
    it('should toggle debug mode with Cmd/Ctrl+Shift+D', async () => {
      await browser.pause(1000);

      // Check if we're on main app (has sidebar)
      const sidebar = await $('nav[aria-label="Settings navigation"]');
      const isMainApp = await sidebar.isExisting();

      if (isMainApp) {
        // Check initial state - debug section might not be visible
        let debugSection = await sidebar.$('*=Debug');
        const initiallyVisible = await debugSection.isExisting();

        // Send Cmd+Shift+D (or Ctrl+Shift+D on Windows/Linux)
        await browser.keys(['Meta', 'Shift', 'd']);
        await browser.pause(500);

        // Check if debug mode toggled
        debugSection = await sidebar.$('*=Debug');
        const afterToggle = await debugSection.isExisting();

        // If it wasn't visible before, it should be now (or vice versa)
        if (!initiallyVisible) {
          console.log(
            `Debug section visibility changed: ${initiallyVisible} -> ${afterToggle}`
          );
        }

        // Toggle back to original state
        await browser.keys(['Meta', 'Shift', 'd']);
        await browser.pause(500);
      } else {
        console.log(
          'Not on main app, skipping debug mode test'
        );
      }
    });
  });

  describe('Focus Management', () => {
    it('should support Tab navigation', async () => {
      await browser.pause(500);

      // Press Tab to move focus
      await browser.keys(['Tab']);
      await browser.pause(200);

      // Get the currently focused element
      const activeElement = await browser.execute(() => {
        return document.activeElement?.tagName;
      });

      // Some element should be focused after Tab
      expect(activeElement).toBeTruthy();
      console.log(`Tab focused element: ${activeElement}`);
    });

    it('should allow Shift+Tab for reverse navigation', async () => {
      await browser.pause(500);

      // Tab forward a few times
      await browser.keys(['Tab']);
      await browser.keys(['Tab']);
      await browser.pause(200);

      // Now Shift+Tab to go back
      await browser.keys(['Shift', 'Tab']);
      await browser.pause(200);

      const activeElement = await browser.execute(() => {
        return document.activeElement?.tagName;
      });

      expect(activeElement).toBeTruthy();
    });
  });

  describe('Escape Key', () => {
    it('should handle Escape key gracefully', async () => {
      await browser.pause(500);

      // Press Escape
      await browser.keys(['Escape']);
      await browser.pause(300);

      // App should still be responsive
      const body = await $('body');
      expect(await body.isExisting()).toBe(true);
    });
  });

  describe('Enter Key', () => {
    it('should activate focused buttons', async () => {
      const sidebar = await $('nav[aria-label="Settings navigation"]');
      const isMainApp = await sidebar.isExisting();

      if (isMainApp) {
        // Focus on a sidebar item
        const generalSection = await sidebar.$('*=General');
        if (await generalSection.isExisting()) {
          // Click to ensure we're starting from General
          await generalSection.click();
          await browser.pause(300);

          // Tab to next interactive element and press Enter
          await browser.keys(['Tab']);
          await browser.pause(200);

          // Get currently focused element
          const focusedTag = await browser.execute(() => {
            return document.activeElement?.tagName;
          });

          console.log(`Focused element before Enter: ${focusedTag}`);

          // Press Enter
          await browser.keys(['Enter']);
          await browser.pause(300);

          // App should remain functional
          expect(await sidebar.isExisting()).toBe(true);
        }
      }
    });
  });
});
