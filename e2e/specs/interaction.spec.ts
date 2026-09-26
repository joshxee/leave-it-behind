import { expect, test } from '@playwright/test';
import { attachShot, collectErrors, gameState, openGame } from './helpers';

test('space scores and arrows move the player', async ({ page }, testInfo) => {
  const errors = collectErrors(page);
  try {
    await openGame(page);
    await page.locator('#bevy-canvas').click();
    const before = await gameState(page);
    await attachShot(page, testInfo, 'before: canvas, SCORE: 0, player square centered', true);

    await page.keyboard.press('Space');
    await page.waitForFunction((s) => window.__bevyState!.score === s + 1, before.score);

    await page.keyboard.down('ArrowRight');
    await page.waitForFunction((x) => window.__bevyState!.player.x > x + 20, before.player.x);
    await page.keyboard.up('ArrowRight');

    const after = await gameState(page);
    expect(after.score).toBe(before.score + 1);
    expect(after.player.x).toBeGreaterThan(before.player.x);
    expect(errors).toEqual([]);
  } finally {
    await attachShot(page, testInfo, 'after-space-and-right: SCORE: 1, player square moved right of center', true);
  }
});

test('score_nine scenario starts one below the threshold', async ({ page }, testInfo) => {
  try {
    await openGame(page, { scenario: 'score_nine' });
    expect((await gameState(page)).score).toBe(9);
  } finally {
    await attachShot(page, testInfo, 'score_nine: SCORE: 9, player square centered', true);
  }
});
