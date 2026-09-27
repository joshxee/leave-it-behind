import { expect, test } from '@playwright/test';
import { attachShot, clickRow, collectErrors, openGame, waitForState } from './helpers';

// The `landing` scenario applies to every run: each level lands 3 s after it starts.

test('landing a level leads to the upgrade screen and on to the next level', async ({ page }, testInfo) => {
  const errors = collectErrors(page);
  try {
    await openGame(page, { scenario: 'landing' });
    const start = await waitForState(page, (s) => s.state === 'Playing');
    expect(start.level).toEqual({ id: 'one', number: 1, name: 'Level 1' });
    const landed = await waitForState(page, (s) => s.state === 'Landed' && s.menu?.screen === 'End');
    expect(landed.menu!.items.map((i) => i.label)).toEqual(['CONTINUE', 'FLY AGAIN (R)', 'MAIN MENU']);
    expect(landed.menu!.focus).toBe(0);

    await page.keyboard.press('Enter');
    const choosing = await waitForState(page, (s) => s.menu?.screen === 'Upgrade');
    expect(choosing.menu!.items.map((i) => i.label)).toEqual(['RUN FASTER', 'FASTER WRENCH', 'WIDER TAPE']);
    await attachShot(page, testInfo, 'upgrade: dimmed ship behind CHOOSE AN UPGRADE, three dim description lines, RUN FASTER highlighted, FASTER WRENCH and WIDER TAPE rows, NEXT: LEVEL 2, 7 FAULTS footer');

    // Pick with the mouse.
    await clickRow(page, 'FASTER WRENCH');
    await waitForState(page, (s) => s.menu?.screen === 'Story');
    await page.keyboard.press('Enter');
    const two = await waitForState(page, (s) => s.state === 'Playing' && s.level.number === 2);
    expect(two.upgrades).toEqual({ runFaster: 0, fasterWrench: 1, widerTape: 0 });
    expect(two.stats.total).toBe(7);
    expect(two.menu).toBeNull();
    await attachShot(page, testInfo, 'level-two: quarters, ALL FIXED - LANDING at the top, LEVEL 2 OF 5 under the room name');

    // Level two lands too: pick with a number key.
    await waitForState(page, (s) => s.state === 'Landed' && s.menu?.screen === 'End');
    await page.keyboard.press('Enter');
    await waitForState(page, (s) => s.menu?.screen === 'Upgrade');
    await page.keyboard.press('3');
    await waitForState(page, (s) => s.menu?.screen === 'Story');
    await page.keyboard.press('Enter');
    const three = await waitForState(page, (s) => s.state === 'Playing' && s.level.number === 3);
    expect(three.upgrades).toEqual({ runFaster: 0, fasterWrench: 1, widerTape: 1 });
    expect(errors).toEqual([]);
  } finally {
    await attachShot(page, testInfo, 'campaign: level three under way');
  }
});

test('landing the last level ends the campaign', async ({ page }, testInfo) => {
  const errors = collectErrors(page);
  try {
    await openGame(page, { scenario: 'final_landing' });
    const start = await waitForState(page, (s) => s.state === 'Playing');
    expect(start.level.number).toBe(5);
    expect(start.upgrades).toEqual({ runFaster: 2, fasterWrench: 1, widerTape: 1 });
    const landed = await waitForState(page, (s) => s.state === 'Landed' && s.menu?.screen === 'End');
    expect(landed.menu!.items.map((i) => i.label)).toEqual(['FLY AGAIN (R)', 'MAIN MENU']);
    expect(errors).toEqual([]);
  } finally {
    await attachShot(page, testInfo, 'final: TOUCHDOWN with Level 5 of 5, ALL 5 LEVELS LANDED. THE SHIP MADE IT HOME. in cyan, FLY AGAIN (R) and MAIN MENU rows');
  }
});
