import { expect, test } from '@playwright/test';
import { attachShot, openGame, step } from './helpers';

// Baselines are Linux-only. Generate/update them with the update-snapshots
// workflow (or the Playwright Docker image), never on Windows/macOS.
// Skipped in CI (--grep-invert @visual) until the first Linux baseline lands.
test('starter scene matches baseline @visual', async ({ page }, testInfo) => {
  try {
    await openGame(page, { freeze: '1' });
    await step(page, 30);
    await expect(page.locator('#bevy-canvas')).toHaveScreenshot('starter-scene.png');
  } finally {
    await attachShot(page, testInfo, 'visual', true);
  }
});
