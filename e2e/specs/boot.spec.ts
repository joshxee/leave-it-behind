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
    expect(s.player.pose).toBe('wrench_hold');
    expect(s.doors.length).toBe(4);
    expect(s.doors.every((d) => d.frame === 0)).toBe(true);
    expect(s.journey.duration).toBe(270);
    expect(s.journey.remaining).toBeGreaterThan(200);
    expect(s.faults).toEqual([]);
    await expect(page.locator('#crash-overlay')).toBeHidden();
    expect(errors).toEqual([]);
  } finally {
    await attachShot(
      page,
      testInfo,
      "boot: tiled quarters, closed doors top and bottom centre, engineer holding the wrench, dark diagnostic screen top left, lockers top right, bunk bottom left, ARRIVAL IN 4:30, belt [1] WRENCH [2] TAPE 20s",
    );
  }
});
