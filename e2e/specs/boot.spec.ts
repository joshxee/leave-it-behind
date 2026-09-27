import { expect, test } from '@playwright/test';
import { attachShot, collectErrors, gameState, openGame, waitForState } from './helpers';

test('boots to the title; the main menu starts level one', async ({ page }, testInfo) => {
  const errors = collectErrors(page);
  try {
    await openGame(page);
    await expect(page.locator('#loading')).toBeHidden();
    const title = await gameState(page);
    expect(title.state).toBe('Menu');
    expect(title.menu?.screen).toBe('Title');
    expect(title.entities.players).toBe(0);
    await attachShot(page, testInfo, 'title: LEAVE IT BEHIND in large letters, CLICK OR PRESS ANY KEY in cyan below, dark background, no HUD');

    await page.keyboard.press('Enter');
    const menu = await waitForState(page, (s) => s.menu?.screen === 'Main');
    // The web build has no Quit: a page cannot close its tab.
    expect(menu.menu!.items.map((i) => i.label)).toEqual(['PLAY', 'HOW TO PLAY', 'SETTINGS']);
    expect(menu.menu!.focus).toBe(0);
    await attachShot(page, testInfo, 'main-menu: LEAVE IT BEHIND title, PLAY row highlighted in cyan, HOW TO PLAY and SETTINGS rows, no QUIT');

    await page.keyboard.press('Enter');
    const s = await waitForState(page, (t) => t.state === 'Playing' && t.entities.players === 1);
    expect(s.menu).toBeNull();
    expect(s.paused).toBe(false);
    expect(s.room).toBe('Quarters');
    expect(s.tool).toBe('Wrench');
    expect(s.player.pose).toBe('wrench_hold');
    expect(s.doors.length).toBe(4);
    expect(s.doors.every((d) => d.frame === 0)).toBe(true);
    expect(s.journey.duration).toBe(270);
    expect(s.journey.remaining).toBeGreaterThan(200);
    // A first-time player's flight waits for the pre-flight check (coach.spec.ts).
    expect(s.journey.launched).toBe(false);
    expect(s.faults).toEqual([]);
    await expect(page.locator('#crash-overlay')).toBeHidden();
    expect(errors).toEqual([]);
  } finally {
    await attachShot(
      page,
      testInfo,
      "boot: tiled quarters, closed doors top and bottom, engineer holding the wrench, dark console top left, ARRIVAL IN 4:30 over the pre-flight check panel, belt [1] WRENCH [2] TAPE 20s",
    );
  }
});
