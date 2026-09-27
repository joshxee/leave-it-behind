import { expect, test } from '@playwright/test';
import { attachShot, averageColor, collectErrors, gameState, openGame, waitForState } from './helpers';

test('the countdown reaching zero lands the ship; R flies again', async ({ page }, testInfo) => {
  const errors = collectErrors(page);
  try {
    await openGame(page, { scenario: 'landing' });
    const start = await gameState(page);
    expect(start.journey.remaining).toBeLessThan(3.5);
    await waitForState(page, (s) => s.state === 'Landed');
    await attachShot(page, testInfo, 'landed: dimmed ship, TOUCHDOWN title, faults fixed and diagnostics stats, "Press R to fly again", timer reads LANDED');
    await page.keyboard.press('r');
    // The landing scenario applies again to the new run: 3 s from arrival.
    const again = await waitForState(page, (s) => s.state === 'Playing');
    expect(again.player.x).toBeCloseTo(start.player.x, 0);
    expect(again.player.y).toBeCloseTo(start.player.y, 0);
    expect(again.tape).toBe(20);
    expect(errors).toEqual([]);
  } finally {
    await attachShot(page, testInfo, 'restarted: quarters again, no end screen');
  }
});

test('an unfixed breach vents the oxygen and ends the run', async ({ page }, testInfo) => {
  const errors = collectErrors(page);
  try {
    await openGame(page, { scenario: 'breach_critical' });
    const lost = await waitForState(page, (s) => s.state === 'Lost');
    expect(lost.stats.failure).toBe('HullBreach');
    expect(errors).toEqual([]);
  } finally {
    await attachShot(page, testInfo, 'lost: red-tinted screen, OXYGEN DEPLETED title, survival time, "Press R to fly again"');
  }
});

test('faults elsewhere only show as a red alarm tint', async ({ page }, testInfo) => {
  const errors = collectErrors(page);
  try {
    await openGame(page, { scenario: 'quiet' });
    const calm = await page.locator('#bevy-canvas').screenshot();
    await page.goto('about:blank');
    await openGame(page, { scenario: 'scramble' });
    // After the start-of-fault jolt, so camera shake cannot move what is in the region.
    const s = await waitForState(page, (t) => t.alarm.active === 3 && t.alarm.jolt === 0);
    expect(s.room).toBe('Quarters');
    const alarmed = await page.locator('#bevy-canvas').screenshot();
    const region = { x: 200, y: 200, w: 200, h: 200 };
    const before = await averageColor(page, calm, region);
    const after = await averageColor(page, alarmed, region);
    expect(after.r - after.b).toBeGreaterThan(before.r - before.b);
    expect(errors).toEqual([]);
  } finally {
    await attachShot(page, testInfo, 'alarm: engineer in the quarters with no fault in sight, whole view washed red by the alarm tint');
  }
});
