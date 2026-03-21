import { test, expect } from '@playwright/test';
import {
  gotoApp,
  openFeaturePanel,
  toggleSettingByHeading,
} from './helpers/settings';

test.describe('Ask AI Settings', () => {
  test.beforeEach(async ({ page }) => {
    await gotoApp(page);
  });

  test('should open the Ask AI feature panel from unified settings', async ({
    page,
  }) => {
    const main = await openFeaturePanel(page, 'ask-ai');
    await expect(main.getByRole('heading', { name: 'Enable Ask AI', exact: true })).toBeVisible();
    await expect(main.getByText('Enable Ask AI to configure its settings.')).toBeVisible();
  });

  test('should reveal shared-provider and conversation settings after enabling Ask AI', async ({
    page,
  }) => {
    const main = await openFeaturePanel(page, 'ask-ai');
    await toggleSettingByHeading(main, 'Enable Ask AI');
    await page.waitForTimeout(300);

    await expect(main.getByText('AI provider is configured centrally')).toBeVisible();
    await expect(main.getByText('Model Override')).toBeVisible();
    await expect(main.getByText('Conversation History')).toBeVisible();
  });

  test('should expose the Ask AI system prompt editor', async ({ page }) => {
    const main = await openFeaturePanel(page, 'ask-ai');
    await toggleSettingByHeading(main, 'Enable Ask AI');
    await page.waitForTimeout(300);

    await expect(main.locator('textarea').first()).toBeVisible();
  });
});
