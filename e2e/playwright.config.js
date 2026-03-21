import { defineConfig, devices } from "@playwright/test";

/**
 * Playwright E2E tests for Handy's browser-mock build.
 *
 * The production Tauri app still requires the native runtime for full E2E,
 * but this config intentionally boots the mocked browser-compatible build so
 * the React app can be validated end to end on any platform.
 */
export default defineConfig({
  testDir: "./playwright",
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: "html",

  use: {
    baseURL: "http://localhost:1420",
    trace: "on-first-retry",
    screenshot: "only-on-failure",
  },

  projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] },
    },
  ],

  // Note: The web server only serves the frontend, which will fail without
  // the Tauri backend unless the browser-compatible E2E Vite config is used.
  webServer: {
    command: "pnpm dev:e2e",
    url: "http://localhost:1420",
    reuseExistingServer: !process.env.CI,
    timeout: 120000,
  },
});
