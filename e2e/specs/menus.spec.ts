import { expect, type Page, test } from '@playwright/test';
import { attachShot, type BevyState, clickRow, collectErrors, gameState, openGame, waitForState } from './helpers';

test('Esc pauses the flight and the clock stops until it resumes', async ({ page }, testInfo) => {
  const errors = collectErrors(page);
  try {
    await openGame(page, { scenario: 'bolts' });
    await page.keyboard.press('Escape');
    const paused = await waitForState(page, (s) => s.paused && s.menu?.screen === 'Pause');
    expect(paused.menu!.items[0].label).toBe('RESUME');
    await attachShot(page, testInfo, 'paused: engine room dimmed behind PAUSED, ARRIVAL IN line, RESUME highlighted, RESTART, HOW TO PLAY, SETTINGS, MAIN MENU rows, no QUIT');
    await page.waitForTimeout(1500);
    const later = await gameState(page);
    expect(later.journey.elapsed).toBe(paused.journey.elapsed);
    expect(later.faults[0].remaining).toBe(paused.faults[0].remaining);

    await page.keyboard.press('Escape');
    await waitForState(page, (s) => !s.paused && s.menu === null);
    await waitForState(page, (s, t) => s.journey.elapsed > t + 0.5, paused.journey.elapsed);
    expect(errors).toEqual([]);
  } finally {
    await attachShot(page, testInfo, 'resumed: engine room with loose orange bolts, no menu, HUD back');
  }
});

/** Presses `key`, then waits for its effect: under software rendering a frame is slow, and keys that land in one frame lose their order. */
async function press<A>(page: Page, key: string, done: (s: BevyState, arg: A) => boolean, arg?: A) {
  await page.keyboard.press(key);
  return waitForState(page, done, arg);
}

test('settings are saved and survive a page reload', async ({ page }, testInfo) => {
  const errors = collectErrors(page);
  try {
    await openGame(page);
    await press(page, 'Space', (s) => s.menu?.screen === 'Main');
    await press(page, 'ArrowDown', (s) => s.menu?.focus === 1);
    await press(page, 'ArrowDown', (s) => s.menu?.focus === 2);
    const settings = await press(page, 'Enter', (s) => s.menu?.screen === 'Settings');
    // The browser owns fullscreen and vsync.
    expect(settings.menu!.items.map((i) => i.label)).toEqual([
      'SCREEN SHAKE 100%',
      'ALARM FLASHING 100%',
      'CONTROLS HINT ON',
      'TIPS ON',
      'PAUSE WHEN UNFOCUSED ON',
      'RESET SETTINGS',
      'RESET PROGRESS',
      'BACK',
    ]);
    await press(page, 'ArrowLeft', (s) => s.settings.shake === 75);
    await press(page, 'ArrowDown', (s) => s.menu?.focus === 1);
    for (const flash of [75, 50, 25, 0]) {
      await press(page, 'ArrowLeft', (s, f) => s.settings.flash === f, flash);
    }
    await attachShot(page, testInfo, 'settings: SETTINGS title, SCREEN SHAKE < 75% >, ALARM FLASHING < 0% > highlighted, CONTROLS HINT ON, TIPS ON, PAUSE WHEN UNFOCUSED ON, RESET SETTINGS, RESET PROGRESS, BACK');

    const stored = await page.evaluate(() => localStorage.getItem('leave-it-behind/settings'));
    expect(stored).toContain('shake: 75');
    await page.reload();
    await page.waitForFunction(() => window.__bevyReady === true, null, { timeout: 90_000 });
    const reloaded = await waitForState(page, (s) => s.ready);
    expect(reloaded.settings.shake).toBe(75);
    expect(reloaded.settings.flash).toBe(0);
    expect(reloaded.notice).toBeNull();
    expect(errors).toEqual([]);
  } finally {
    await attachShot(page, testInfo, 'reloaded: title screen again after the reload');
  }
});

test('the mouse highlights and clicks menu rows', async ({ page }, testInfo) => {
  const errors = collectErrors(page);
  try {
    await openGame(page);
    await page.mouse.click(640, 600);
    await waitForState(page, (s) => s.menu?.screen === 'Main');
    await clickRow(page, 'SETTINGS');
    await waitForState(page, (s) => s.menu?.screen === 'Settings');
    await clickRow(page, 'SCREEN SHAKE 100%');
    await waitForState(page, (s) => s.settings.shake === 0 && s.menu?.focus === 0);
    await attachShot(page, testInfo, 'clicked-setting: SETTINGS with SCREEN SHAKE < 0% > highlighted under the mouse');
    await clickRow(page, 'BACK');
    await waitForState(page, (s) => s.menu?.screen === 'Main');
    await clickRow(page, 'PLAY');
    await waitForState(page, (s) => s.state === 'Playing' && s.entities.players === 1);
    expect(errors).toEqual([]);
  } finally {
    await attachShot(page, testInfo, 'played: the flight started from a mouse click, quarters on screen, the pre-flight check panel under ARRIVAL IN 2:30');
  }
});

test('losing focus pauses the flight', async ({ page }, testInfo) => {
  const errors = collectErrors(page);
  try {
    await openGame(page, { scenario: 'quiet' });
    await page.evaluate(() => document.getElementById('bevy-canvas')!.blur());
    await waitForState(page, (s) => s.paused && s.menu?.screen === 'Pause');
    expect(errors).toEqual([]);
  } finally {
    await attachShot(page, testInfo, 'unfocused: quarters dimmed behind the PAUSED menu');
  }
});

test('the end screen scores the flight and leads back to the main menu', async ({ page }, testInfo) => {
  const errors = collectErrors(page);
  try {
    await openGame(page, { scenario: 'landing' });
    const landed = await waitForState(page, (s) => s.state === 'Landed' && s.menu?.screen === 'End');
    expect(landed.lastRun!.newBest).toBe(true);
    expect(landed.lastRun!.record.landed).toBe(true);
    expect(landed.menu!.items.map((i) => i.label)).toEqual(['CONTINUE', 'FLY AGAIN (R)', 'MAIN MENU']);
    await page.keyboard.press('Escape');
    const menu = await waitForState(page, (s) => s.state === 'Menu' && s.menu?.screen === 'Main');
    expect(menu.progress.flights).toBe(1);
    expect(menu.progress.best!.landed).toBe(true);
    expect(errors).toEqual([]);
  } finally {
    await attachShot(page, testInfo, 'main-menu-progress: main menu with a dim line under the rows: LEVELS LANDED: 1 OF 5 - 1 FLIGHT');
  }
});
