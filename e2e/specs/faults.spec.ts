import { expect, test } from '@playwright/test';
import { aimAt, attachShot, collectErrors, gameState, openGame, toScreen, waitForState } from './helpers';

test('wrench: click each loose bolt to fix the engine', async ({ page }, testInfo) => {
  const errors = collectErrors(page);
  try {
    await openGame(page, { scenario: 'bolts' });
    const start = await waitForState(page, (s) => s.looseBolts.length === 3 && s.snap);
    await attachShot(page, testInfo, 'bolts-loose: upper engine with three orange bolts sticking out of its lower face, wrench head green (snapped) on the left one');
    const bolts = [...start.looseBolts].sort((a, b) => a.x - b.x);
    for (const [i, bolt] of bolts.entries()) {
      const s = await gameState(page);
      if (Math.abs(bolt.x - s.player.x) > 4) {
        const key = bolt.x > s.player.x ? 'd' : 'a';
        await page.keyboard.down(key);
        await waitForState(page, (t, x) => Math.abs(x - t.player.x) <= 20, bolt.x);
        await page.keyboard.up(key);
      }
      await aimAt(page, bolt);
      const aimed = await waitForState(page, (t) => t.snap);
      const at = toScreen(aimed, bolt);
      await page.mouse.click(at.x, at.y);
      await waitForState(page, (t, n) => t.looseBolts.length === n, bolts.length - i - 1);
    }
    const done = await waitForState(page, (s) => s.faults.length === 0);
    expect(done.stats.fixed).toBe(1);
    expect(errors).toEqual([]);
  } finally {
    await attachShot(page, testInfo, 'bolts-fixed: all engine bolts grey and flush, engine block grey (not glowing)');
  }
});

test('tape: hold the button on the breach to seal it', async ({ page }, testInfo) => {
  const errors = collectErrors(page);
  try {
    await openGame(page, { scenario: 'breach' });
    const s = await gameState(page);
    const breach = s.faults[0];
    expect(breach.kind).toBe('HullBreach');
    expect(s.room).toBe('Airlock');
    await aimAt(page, breach);
    await attachShot(page, testInfo, 'breach-open: airlock (purple floor), black hole in the upper wall with a white ring of escaping air, tape roll held under it');
    await page.mouse.down();
    await waitForState(page, (t) => t.faults.length === 0);
    await page.mouse.up();
    const done = await gameState(page);
    expect(done.stats.fixed).toBe(1);
    expect(done.tape).toBeLessThan(18);
    expect(done.entities.tapeStrips).toBeGreaterThan(5);
    expect(errors).toEqual([]);
  } finally {
    await attachShot(page, testInfo, 'breach-sealed: tan tape strips over the spot, no hole or air ring left');
  }
});

test('helm: E locks in, WASD steers the marker into the centre band', async ({ page }, testInfo) => {
  const errors = collectErrors(page);
  try {
    await openGame(page, { scenario: 'drift' });
    await waitForState(page, (s) => s.focus === 'Helm' && !s.nav.inBand);
    await attachShot(page, testInfo, 'drift: cockpit, nav marker off-centre and red, display frame blinking red');
    await page.keyboard.press('e');
    await waitForState(page, (s) => s.nav.engaged && s.player.locked);

    // Closed-loop steering: hold the keys that point back at the centre.
    const held = new Set<string>();
    const deadline = Date.now() + 90_000;
    let shotTaken = false;
    for (;;) {
      const s = await gameState(page);
      if (s.faults.length === 0) break;
      if (Date.now() > deadline) throw new Error(`drift not fixed: ${JSON.stringify(s.nav)}`);
      if (!shotTaken && s.nav.inBand && s.faults[0].repair > 0.3) {
        shotTaken = true;
        await attachShot(page, testInfo, 'steering: marker green inside the centre band, green hold bar growing along the display bottom');
      }
      const want = new Set<string>();
      if (s.nav.x > 0.05) want.add('a');
      if (s.nav.x < -0.05) want.add('d');
      if (s.nav.y > 0.05) want.add('s');
      if (s.nav.y < -0.05) want.add('w');
      for (const k of held) if (!want.has(k)) { await page.keyboard.up(k); held.delete(k); }
      for (const k of want) if (!held.has(k)) { await page.keyboard.down(k); held.add(k); }
      await page.waitForTimeout(100);
    }
    for (const k of held) await page.keyboard.up(k);
    const done = await waitForState(page, (s) => !s.nav.engaged);
    expect(done.stats.fixed).toBe(1);
    expect(done.player.locked).toBe(false);
    expect(errors).toEqual([]);
  } finally {
    await attachShot(page, testInfo, 'drift-fixed: marker settling to the centre, display frame grey, engineer free');
  }
});

test('diagnostics: E at the console pinpoints every fault', async ({ page }, testInfo) => {
  const errors = collectErrors(page);
  try {
    await openGame(page, { scenario: 'diagnostics' });
    await waitForState(page, (s) => s.focus === 'Diagnostics');
    await page.keyboard.press('e');
    await waitForState(page, (s) => s.diag === 'Scanning');
    const open = await waitForState(page, (s) => s.diag === 'Open');
    expect(open.diagUses).toBe(1);
    await attachShot(page, testInfo, 'diagnostics: minimap of all five rooms, three red markers on the upper engine, one on the airlock upper wall, orange dot for the engineer in the quarters, list of two faults with seconds left');
    await page.keyboard.press('e');
    await waitForState(page, (s) => s.diag === 'Closed');
    expect(errors).toEqual([]);
  } finally {
    await attachShot(page, testInfo, 'diagnostics-closed: quarters again, no minimap, faint red alarm tint');
  }
});
