import { expect, test } from '@playwright/test';
import { aimAt, attachShot, collectErrors, gameState, openGame, toScreen, waitForState } from './helpers';

test('WASD walks the engineer, doors slide open, the camera cuts room to room', async ({ page }, testInfo) => {
  const errors = collectErrors(page);
  try {
    await openGame(page, { scenario: 'quiet' });
    const start = await gameState(page);
    expect(start.room).toBe('Quarters');

    // Onto the conduit that runs through every door, then forward.
    await page.keyboard.down('d');
    await waitForState(page, (s) => s.player.x >= -6);
    await page.keyboard.up('d');
    await page.keyboard.down('w');
    const walking = await waitForState(page, (s) => s.player.walking && s.doors.some((d) => d.frame > 0));
    expect(walking.player.pose).toBe('wrench_walk');
    await attachShot(page, testInfo, 'walking: engineer walking up the conduit with the wrench held out ahead, the sliding door above sliding open');
    const cockpit = await waitForState(page, (s) => s.room === 'Cockpit');
    await page.keyboard.up('w');
    expect(cockpit.player.y).toBeGreaterThan(start.player.y);
    expect(cockpit.camera.y).toBeGreaterThan(start.camera.y);
    await attachShot(page, testInfo, 'cockpit: window and consoles along the top wall, pilot seat with a cyan joystick, blue nav display (marker centred) below it, engineer just inside the bottom door, no other room visible');

    await page.keyboard.down('s');
    await waitForState(page, (s) => s.room === 'Engine');
    await page.keyboard.up('s');
    const idle = await waitForState(page, (s) => !s.player.walking);
    expect(idle.player.pose).toBe('wrench_hold');
    expect(errors).toEqual([]);
  } finally {
    await attachShot(page, testInfo, 'engine-room: two dark engine blocks left and right of the conduit with grey bolts on their long sides, grated floor, engineer near the top door');
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
    expect(up.player.dir).toBe(12);

    await page.keyboard.press('2');
    await waitForState(page, (t) => t.tool === 'Tape' && t.player.pose === 'tape_hold');
    await attachShot(page, testInfo, 'tape-held: engineer holding the tape roll up (toward the mouse), belt highlights [2] TAPE');

    await page.mouse.wheel(0, 120);
    await waitForState(page, (t) => t.tool === 'Wrench');
    await page.keyboard.press('2');
    await waitForState(page, (t) => t.tool === 'Tape');
    await page.keyboard.press('1');
    await waitForState(page, (t) => t.tool === 'Wrench' && t.player.pose === 'wrench_hold');
    expect(errors).toEqual([]);
  } finally {
    await attachShot(page, testInfo, 'wrench-held: engineer holding the wrench up toward the mouse, belt highlights [1] WRENCH');
  }
});

test('tools stay in hand while walking but only work standing still', async ({ page }, testInfo) => {
  const errors = collectErrors(page);
  try {
    await openGame(page, { scenario: 'bolts' });
    const start = await waitForState(page, (s) => s.snap && s.looseBolts.length === 3);
    const bolts = [...start.looseBolts].sort((a, b) => a.y - b.y);
    // Walk along the panel and click on the way: nothing turns.
    await page.keyboard.down('s');
    const walking = await waitForState(page, (s) => s.player.walking);
    expect(walking.player.pose).toBe('wrench_walk');
    expect(walking.snap).toBe(false);
    const at = toScreen(walking, bolts[0]);
    await page.mouse.click(at.x, at.y);
    await page.waitForTimeout(300);
    expect((await gameState(page)).turning).toBe(false);
    await page.keyboard.up('s');
    // Standing again: the wrench snaps to a bolt and a click turns it.
    const standing = await waitForState(page, (s) => !s.player.walking && s.player.pose === 'wrench_hold');
    const nearest = [...standing.looseBolts].sort(
      (a, b) => Math.hypot(a.x - standing.player.x, a.y - standing.player.y) - Math.hypot(b.x - standing.player.x, b.y - standing.player.y),
    )[0];
    await aimAt(page, nearest);
    const aimed = await waitForState(page, (s) => s.snap);
    const target = toScreen(aimed, nearest);
    await page.mouse.click(target.x, target.y);
    await waitForState(page, (s) => s.turning && s.player.pose === 'wrench_use');
    await attachShot(page, testInfo, 'wrench-turn: engineer standing at the engine, wrench jaw on an orange bolt inside a cyan ring, turning it');
    await waitForState(page, (s) => s.looseBolts.length === 2);
    expect(errors).toEqual([]);
  } finally {
    await attachShot(page, testInfo, 'one-bolt-down: two orange bolts left on the engine, the turned one grey');
  }
});
