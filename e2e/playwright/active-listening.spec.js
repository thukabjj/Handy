import { test, expect } from '@playwright/test';
import {
  gotoApp,
  openFeaturePanel,
  toggleSettingByHeading,
} from './helpers/settings';

test.describe('Active Listening Settings', () => {
  test.beforeEach(async ({ page }) => {
    await gotoApp(page);
  });

  test('should open the Active Listening feature panel from unified settings', async ({
    page,
  }) => {
    const main = await openFeaturePanel(page, 'active-listening');
    await expect(
      main.getByRole('heading', { name: 'Enable Active Listening', exact: true }),
    ).toBeVisible();
    await expect(main.getByText('Enable Active Listening to configure its settings.')).toBeVisible();
  });

  test('should reveal advanced sections after enabling Active Listening', async ({
    page,
  }) => {
    const main = await openFeaturePanel(page, 'active-listening');
    await toggleSettingByHeading(main, 'Enable Active Listening');
    await page.waitForTimeout(300);

    await expect(main.getByText('AI provider is configured centrally')).toBeVisible();
    await expect(
      main.getByRole('heading', { name: 'Screen Vision', exact: true }),
    ).toBeVisible();
    await expect(
      main.getByRole('heading', { name: 'Session History', exact: true }),
    ).toBeVisible();
  });

  test('should expose prompt management after enabling Active Listening', async ({
    page,
  }) => {
    const main = await openFeaturePanel(page, 'active-listening');
    await toggleSettingByHeading(main, 'Enable Active Listening');
    await page.waitForTimeout(300);

    await expect(main.getByRole('heading', { name: 'Prompts', exact: true })).toBeVisible();
    await expect(main.getByRole('button', { name: 'New Prompt' })).toBeVisible();
  });
});
