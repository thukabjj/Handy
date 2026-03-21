import { test, expect } from '@playwright/test';
import {
  gotoApp,
  openFeaturePanel,
  toggleSettingByHeading,
} from './helpers/settings';

async function enableActiveListening(main, page) {
  await toggleSettingByHeading(main, 'Enable Active Listening');
  await page.waitForTimeout(300);
}

test.describe('Screen Vision (macOS mock)', () => {
  test.beforeEach(async ({ page }) => {
    await gotoApp(page);
  });

  test('should render screen vision controls with OpenRouter defaults', async ({
    page,
  }) => {
    const main = await openFeaturePanel(page, 'active-listening');
    await enableActiveListening(main, page);

    await expect(main.getByRole('heading', { name: 'Screen Vision', exact: true })).toBeVisible();
    await expect(main.getByRole('heading', { name: 'Vision Provider', exact: true })).toBeVisible();
    await expect(main.getByText('OpenRouter')).toBeVisible();
  });

  test('should save the Screen Vision API key and model', async ({ page }) => {
    const main = await openFeaturePanel(page, 'active-listening');
    await enableActiveListening(main, page);

    const apiKeyInput = main.locator('input[placeholder="sk-..."]').last();
    const modelInput = main
      .locator('input[placeholder="qwen/qwen2.5-vl-72b-instruct:free"]')
      .first();

    await expect(apiKeyInput).toBeVisible();
    await expect(modelInput).toBeVisible();

    await apiKeyInput.fill('sk-or-test-key');
    await apiKeyInput.blur();
    await modelInput.fill('qwen/qwen2.5-vl-72b-instruct:free');
    await modelInput.blur();

    await expect(apiKeyInput).toHaveValue('sk-or-test-key');
    await expect(modelInput).toHaveValue('qwen/qwen2.5-vl-72b-instruct:free');
  });

  test('should run the screen vision test-once and timed session actions', async ({
    page,
  }) => {
    const main = await openFeaturePanel(page, 'active-listening');
    await enableActiveListening(main, page);

    await toggleSettingByHeading(main, 'Enable Screen Vision');
    await page.waitForTimeout(250);

    await main.getByRole('button', { name: 'Test once' }).click();
    await expect(main.getByText('Latest Analysis')).toBeVisible();

    await main.getByRole('button', { name: 'Start timed session' }).click();
    await expect(main.getByText('Status: running')).toBeVisible();

    await main.getByRole('button', { name: 'Stop' }).click();
    await expect(main.getByText('Status: idle')).toBeVisible();
  });
});
