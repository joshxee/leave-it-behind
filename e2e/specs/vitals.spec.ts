import { expect, test } from '@playwright/test';
import { attachShot, collectErrors, gameState, openGame, step } from './helpers';

test('the vitals panel reads calm on a quiet ship', async ({ page }, testInfo) => {
  const errors = collectErrors(page);
  try {
    await openGame(page, { freeze: '1', scenario: 'quiet' });
    await step(page, 30);
    const s = await gameState(page);
    expect(s.vitals.gauges).toEqual([
      { label: 'O2', value: '100%', alert: false },
      { label: 'HEAT', value: '0%', alert: false },
      { label: 'COURSE', value: 'OK', alert: false },
    ]);
    expect(errors).toEqual([]);
  } finally {
    await attachShot(page, testInfo, 'vitals calm: top-right panel in cold colours, O2 bar full at 100%, HEAT bar empty at 0%, COURSE OK');
  }
});

test('every failing system shows in the vitals panel, with the time to impact', async ({ page }, testInfo) => {
  const errors = collectErrors(page);
  try {
    await openGame(page, { freeze: '1', scenario: 'scramble' });
    await step(page, 60);
    const s = await gameState(page);
    const [o2, heat, course] = s.vitals.gauges;
    expect(s.vitals.oxygen).toBeLessThan(1);
    expect(s.vitals.heat).toBeGreaterThan(0);
    expect([o2.label, heat.label, course.label]).toEqual(['O2', 'HEAT', 'IMPACT IN']);
    expect([o2.alert, heat.alert, course.alert]).toEqual([true, true, true]);
    expect(course.value).toMatch(/^0:\d\d$/);
    const drift = s.faults.find((f) => f.kind === 'TrajectoryDrift')!;
    expect(Math.abs(parseInt(course.value.slice(2), 10) - drift.remaining)).toBeLessThanOrEqual(1);
    expect(errors).toEqual([]);
  } finally {
    await attachShot(page, testInfo, 'vitals scramble: top-right panel all red, O2 just under 100%, HEAT just above 0%, IMPACT IN counting down from 0:50');
  }
});

test('a breach on low air drains what is left', async ({ page }, testInfo) => {
  const errors = collectErrors(page);
  try {
    await openGame(page, { freeze: '1', scenario: 'second_breach' });
    await step(page, 10);
    const before = await gameState(page);
    await step(page, 120);
    const after = await gameState(page);
    // Two seconds of a 50 s breach: 4% of a full tank.
    expect(before.vitals.oxygen - after.vitals.oxygen).toBeCloseTo(0.04, 2);
    expect(after.vitals.oxygen).toBeLessThan(0.4);
    const [o2, heat, course] = after.vitals.gauges;
    expect(o2.alert).toBe(true);
    expect(Math.abs(parseInt(o2.value, 10) - after.vitals.oxygen * 100)).toBeLessThanOrEqual(1);
    expect([heat.alert, course.alert]).toEqual([false, false]);
    expect(errors).toEqual([]);
  } finally {
    await attachShot(page, testInfo, 'vitals low air: airlock breach venting, top-right O2 bar about a third full, red, reading under 40%, HEAT 0% and COURSE OK in cold colours');
  }
});
