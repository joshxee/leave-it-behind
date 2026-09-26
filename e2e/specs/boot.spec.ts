import { expect, test } from '@playwright/test';
import { attachShot, collectErrors, gameState, openGame } from './helpers';

test('boots into level one with no errors', async ({ page }, testInfo) => {
  const errors = collectErrors(page);
  try {
    await openGame(page);
    const s = await gameState(page);
    expect(s.state).toBe('Playing');
    expect(s.entities.players).toBe(1);
    expect(s.room).toBe('Quarters');
    expect(s.tool).toBe('Wrench');
    expect(s.journey.duration).toBe(240);
    expect(s.journey.remaining).toBeGreaterThan(200);
    expect(s.faults).toEqual([]);
    await expect(page.locator('#crash-overlay')).toBeHidden();
    expect(errors).toEqual([]);
  } finally {
    await attachShot(
      page,
      testInfo,
      "boot: engineer's quarters (teal floor), orange engineer on the spine holding the wrench, diagnostic console on the upper wall, bunk lower left, ARRIVAL IN 4:00 top centre, belt [1] WRENCH [2] TAPE 20s bottom",
    );
  }
});
