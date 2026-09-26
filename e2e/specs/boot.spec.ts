import { expect, test } from '@playwright/test';
import { attachShot, collectErrors, gameState, openGame } from './helpers';

test('boots with no errors and reaches Playing', async ({ page }, testInfo) => {
  const errors = collectErrors(page);
  try {
    await openGame(page);
    const state = await gameState(page);
    expect(state.state).toBe('Playing');
    expect(state.entities.players).toBe(1);
    await expect(page.locator('#crash-overlay')).toBeHidden();
    expect(errors).toEqual([]);
  } finally {
    await attachShot(page, testInfo, 'boot');
  }
});
