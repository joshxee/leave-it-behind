import { expect, test } from '@playwright/test';
import { collectErrors, distinctColors } from './helpers';

// Needs no test bridge, so it also runs against a real release build.
test('release build runs for 20s without crashing @release-smoke', async ({ page }, testInfo) => {
  test.setTimeout(150_000);
  const errors = collectErrors(page);
  let shot: Buffer | undefined;
  try {
    await page.goto('/index.html');
    await page.waitForTimeout(20_000);
    await expect(page.locator('#crash-overlay')).toBeHidden();
    expect(errors).toEqual([]);
    shot = await page.locator('#bevy-canvas').screenshot();
    expect(await distinctColors(page, shot, 2), 'canvas should not be a single solid color').toBeGreaterThan(1);
  } finally {
    await testInfo.attach('release-smoke: canvas after 20s, game rendering (not a solid color), no crash overlay', {
      body: shot ?? (await page.screenshot({ fullPage: true })),
      contentType: 'image/png',
    });
  }
});
