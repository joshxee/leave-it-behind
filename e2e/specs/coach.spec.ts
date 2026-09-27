import { expect, type Page, test } from '@playwright/test';
import {
  attachShot,
  type BevyState,
  collectErrors,
  gameState,
  openGame,
  turnLooseBolts,
  waitForState,
} from './helpers';

/** Presses `key`, then waits for its effect (keys that land in one slow frame lose their order). */
async function press<A>(page: Page, key: string, done: (s: BevyState, arg: A) => boolean, arg?: A) {
  await page.keyboard.press(key);
  return waitForState(page, done, arg);
}

/** From the title to a flight, as a player gets there: any key, then PLAY. */
async function playFromTheTitle(page: Page) {
  await press(page, 'Enter', (s) => s.menu?.screen === 'Main');
  return press(page, 'Enter', (s) => s.state === 'Playing' && s.entities.players === 1);
}

/** Walks along x, then y, until the diagnostic console is in reach. */
async function walkToTheConsole(page: Page) {
  const s = await gameState(page);
  if (Math.abs(s.console.x - s.player.x) > 8) {
    const key = s.console.x > s.player.x ? 'd' : 'a';
    await page.keyboard.down(key);
    await waitForState(page, (t) => Math.abs(t.console.x - t.player.x) <= 8);
    await page.keyboard.up(key);
  }
  if ((await gameState(page)).focus !== 'Diagnostics') {
    const key = s.console.y > s.player.y ? 'w' : 's';
    await page.keyboard.down(key);
    await waitForState(page, (t) => t.focus === 'Diagnostics');
    await page.keyboard.up(key);
  }
  await waitForState(page, (t) => !t.player.walking && t.focus === 'Diagnostics');
}

test('a first flight waits for the pre-flight check, and only the first', async ({ page }, testInfo) => {
  const errors = collectErrors(page);
  try {
    // Each test gets a fresh localStorage: this is a first-time player.
    await openGame(page);
    const held = await playFromTheTitle(page);
    expect(held.journey.launched).toBe(false);
    expect(held.coach.lines).toHaveLength(1);
    expect(held.coach.lines[0]).toContain('diagnostic console');
    expect(held.settings.tips).toBe(true);
    await attachShot(page, testInfo, 'preflight: quarters, ARRIVAL IN 4:30, a cyan-bordered panel under it reading Pre-flight check: walk to the diagnostic console and press E, controls hint at the bottom');
    await page.waitForTimeout(1000);
    const later = await gameState(page);
    expect(later.journey.elapsed).toBe(0);
    expect(later.faults).toEqual([]);

    await walkToTheConsole(page);
    await press(page, 'e', (s) => s.diag === 'Scanning');
    const open = await waitForState(page, (s) => s.diag === 'Open' && s.journey.launched);
    expect(open.coach.lines).toEqual([expect.stringContaining('Cleared for launch')]);
    await attachShot(page, testInfo, 'launched: diagnostic screen open reading ALL SYSTEMS NOMINAL, above it the panel reading Cleared for launch. When the alarm goes off, check this console to find the fault.');
    await waitForState(page, (s) => s.journey.elapsed > 0.5);
    const stored = await page.evaluate(() => localStorage.getItem('leave-it-behind/settings'));
    expect(stored).toContain('preflight: true');

    // Once only: after a reload, the next flight goes straight to the countdown.
    await page.reload();
    await page.waitForFunction(() => window.__bevyReady === true, null, { timeout: 90_000 });
    await waitForState(page, (s) => s.ready && s.menu?.screen === 'Title');
    await page.locator('#bevy-canvas').focus();
    const again = await playFromTheTitle(page);
    expect(again.journey.launched).toBe(true);
    expect(again.coach.lines).toEqual([]);
    // The clock rounds up: it reads 4:29 once 1 s has gone.
    await waitForState(page, (s) => s.journey.elapsed > 1.5);
    expect(errors).toEqual([]);
  } finally {
    await attachShot(page, testInfo, 'second-flight: quarters with no coaching panel, ARRIVAL IN already below 4:30');
  }
});

test('the first loose bolts bring up a tip until they are fixed', async ({ page }, testInfo) => {
  const errors = collectErrors(page);
  try {
    await openGame(page, { scenario: 'first_bolts' });
    const s = await waitForState(page, (t) => t.looseBolts.length === 3 && t.snap && t.coach.lines.length === 1);
    expect(s.journey.launched).toBe(true);
    expect(s.coach.lines[0]).toBe('Use the wrench [1] to screw the bolts back into the engine.');
    await attachShot(page, testInfo, 'bolts-tip: engine room, three orange bolts on the left engine, engineer beside them with the wrench, panel under the timer reading Use the wrench [1] to screw the bolts back into the engine.');
    await turnLooseBolts(page);
    const done = await waitForState(page, (t) => t.faults.length === 0);
    expect(done.coach.lines).toEqual([]);
    expect(done.stats.fixed).toBe(1);
    expect(errors).toEqual([]);
  } finally {
    await attachShot(page, testInfo, 'bolts-tip-gone: engine room with every bolt grey and flush, no coaching panel');
  }
});

test('TIPS in the settings switches the coaching off and back on', async ({ page }, testInfo) => {
  const errors = collectErrors(page);
  try {
    await openGame(page);
    await press(page, 'Enter', (s) => s.menu?.screen === 'Main');
    await press(page, 'ArrowDown', (s) => s.menu?.focus === 1);
    await press(page, 'ArrowDown', (s) => s.menu?.focus === 2);
    await press(page, 'Enter', (s) => s.menu?.screen === 'Settings');
    for (const row of [1, 2, 3]) await press(page, 'ArrowDown', (s, r) => s.menu?.focus === r, row);
    const on = await gameState(page);
    expect(on.menu!.items[3].label).toBe('TIPS ON');
    await press(page, 'Enter', (s) => !s.settings.tips);
    await waitForState(page, (s) => s.menu?.items[3].label === 'TIPS OFF');
    await attachShot(page, testInfo, 'tips-off: SETTINGS with TIPS < OFF > highlighted between CONTROLS HINT ON and PAUSE WHEN UNFOCUSED ON');

    // Off: the flight launches at once, with no coaching.
    await press(page, 'Escape', (s) => s.menu?.screen === 'Main');
    for (const row of [1, 0]) await press(page, 'ArrowUp', (s, r) => s.menu?.focus === r, row);
    const flight = await press(page, 'Enter', (s) => s.state === 'Playing' && s.entities.players === 1);
    expect(flight.journey.launched).toBe(true);
    expect(flight.coach.lines).toEqual([]);

    // Back on from the pause menu: the next flight waits for the check again.
    await press(page, 'Escape', (s) => s.paused && s.menu?.screen === 'Pause');
    for (const row of [1, 2, 3]) await press(page, 'ArrowDown', (s, r) => s.menu?.focus === r, row);
    await press(page, 'Enter', (s) => s.menu?.screen === 'Settings');
    for (const row of [1, 2, 3]) await press(page, 'ArrowDown', (s, r) => s.menu?.focus === r, row);
    await press(page, 'Enter', (s) => s.settings.tips);
    // Back on the pause menu, SETTINGS is still highlighted: up to RESTART.
    await press(page, 'Escape', (s) => s.menu?.screen === 'Pause' && s.menu.focus === 3);
    for (const row of [2, 1]) await press(page, 'ArrowUp', (s, r) => s.menu?.focus === r, row);
    await press(page, 'Enter', (s) => s.menu?.screen === 'ConfirmRestart');
    await press(page, 'ArrowDown', (s) => s.menu?.focus === 1);
    const restarted = await press(page, 'Enter', (s) => s.state === 'Playing' && s.menu === null && !s.paused);
    expect(restarted.journey.launched).toBe(false);
    expect(restarted.coach.lines[0]).toContain('diagnostic console');
    expect(errors).toEqual([]);
  } finally {
    await attachShot(page, testInfo, 'tips-on-again: quarters after a restart, ARRIVAL IN 4:30 and the pre-flight check panel again');
  }
});
