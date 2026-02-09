import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright E2E tests for Handy
 *
 * IMPORTANT: Tauri apps require the Tauri runtime to function. The frontend
 * cannot run standalone in a browser because it depends on @tauri-apps/api
 * which only works inside the Tauri webview.
 *
 * For E2E testing, you have these options:
 *
 * 1. **tauri-driver (Linux/Windows only)** - Use WebdriverIO + tauri-driver
 *    to test the actual built application. Not supported on macOS.
 *
 * 2. **Component testing with mocks** - The existing Vitest unit tests mock
 *    Tauri APIs and test components in isolation. This is the recommended
 *    approach for testing UI logic.
 *
 * 3. **Manual testing** - Run `bun run tauri dev` and manually test the app.
 *
 * This Playwright config is kept for reference but won't work for Tauri apps
 * that depend on native APIs. Use the WebdriverIO setup (e2e:wdio) on
 * Linux/Windows for true E2E testing.
 */
export default defineConfig({
  testDir: './playwright',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: 'html',

  use: {
    // Note: This URL won't work for Tauri apps - frontend needs Tauri runtime
    baseURL: 'http://localhost:1420',
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
  },

  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],

  // Note: The web server only serves the frontend, which will fail without
  // the Tauri backend. These tests are for reference only.
  webServer: {
    command: 'bun run dev',
    url: 'http://localhost:1420',
    reuseExistingServer: !process.env.CI,
    timeout: 120000,
  },
});
