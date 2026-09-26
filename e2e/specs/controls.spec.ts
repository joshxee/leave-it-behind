import { expect, test } from '@playwright/test';
import { aimAt, attachShot, collectErrors, gameState, openGame, waitForState } from './helpers';

test('WASD walks the engineer and the camera cuts room to room', async ({ page }, testInfo) => {
  const errors = collectErrors(page);
  try {
    await openGame(page, { scenario: 'quiet' });
    const start = await gameState(page);
    expect(start.room).toBe('Quarters');

    await page.keyboard.down('d');
    const cockpit = await waitForState(page, (s) => s.room === 'Cockpit');
    await page.keyboard.up('d');
    expect(cockpit.player.x).toBeGreaterThan(start.player.x);
    expect(cockpit.camera.x).toBeGreaterThan(start.camera.x);
    await attachShot(page, testInfo, 'cockpit: dark blue floor, nav display (green bands, marker centred) on the right, red joystick, blue window in the front wall, no other room visible');

    await page.keyboard.down('a');
    await waitForState(page, (s) => s.room === 'Engine');
    await page.keyboard.up('a');
    expect(errors).toEqual([]);
  } finally {
    await attachShot(page, testInfo, 'engine-room: brown floor, two grey engine blocks above and below the spine with grey bolts, engineer near the right door');
  }
});

test('the mouse aims; number keys and the wheel switch tools', async ({ page }, testInfo) => {
  const errors = collectErrors(page);
  try {
    await openGame(page, { scenario: 'quiet' });
    const s = await gameState(page);
    await aimAt(page, { x: s.player.x, y: s.player.y + 150 });
    const up = await waitForState(page, (t) => t.player.fy > 0.95);
    expect(Math.abs(up.player.fx)).toBeLessThan(0.1);

    await page.keyboard.press('2');
    await waitForState(page, (t) => t.tool === 'Tape');
    await attachShot(page, testInfo, 'tape-held: tan tape roll above the engineer (aimed up), belt highlights [2] TAPE');

    await page.mouse.wheel(0, 120);
    await waitForState(page, (t) => t.tool === 'Wrench');
    await page.keyboard.press('2');
    await waitForState(page, (t) => t.tool === 'Tape');
    await page.keyboard.press('1');
    await waitForState(page, (t) => t.tool === 'Wrench');
    expect(errors).toEqual([]);
  } finally {
    await attachShot(page, testInfo, 'wrench-held: steel wrench pointing up from the engineer, belt highlights [1] WRENCH');
  }
});
