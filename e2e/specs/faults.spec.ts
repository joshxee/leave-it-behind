import { expect, type Page, test } from '@playwright/test';
import { aimAt, attachShot, type BevyState, collectErrors, gameState, openGame, toScreen, waitForState } from './helpers';

test('wrench: click each loose bolt to fix the engine', async ({ page }, testInfo) => {
  const errors = collectErrors(page);
  try {
    await openGame(page, { scenario: 'bolts' });
    const start = await waitForState(page, (s) => s.looseBolts.length === 3 && s.snap);
    await attachShot(page, testInfo, 'bolts-loose: left engine with three exposed threaded bolts on its spine side, engineer beside the lowest one with the wrench jaw on it inside a cyan ring');
    // The panel runs along the engine's side: walk along it from bolt to bolt.
    const first = start.looseBolts[0];
    const last = start.looseBolts[start.looseBolts.length - 1];
    const vertical = Math.abs(last.y - first.y) > Math.abs(last.x - first.x);
    const along = (p: { x: number; y: number }) => (vertical ? p.y : p.x);
    const bolts = [...start.looseBolts].sort((a, b) => along(a) - along(b));
    for (const [i, bolt] of bolts.entries()) {
      const s = await gameState(page);
      if (Math.abs(along(bolt) - along(s.player)) > 4) {
        const ahead = along(bolt) > along(s.player);
        const key = vertical ? (ahead ? 'w' : 's') : ahead ? 'd' : 'a';
        await page.keyboard.down(key);
        await waitForState(
          page,
          (t, a) => Math.abs(a.target - (a.vertical ? t.player.y : t.player.x)) <= 20,
          { target: along(bolt), vertical },
        );
        await page.keyboard.up(key);
        await waitForState(page, (t) => !t.player.walking);
      }
      await aimAt(page, bolt);
      const aimed = await waitForState(page, (t) => t.snap);
      const at = toScreen(aimed, bolt);
      await page.mouse.click(at.x, at.y);
      if (i === 0) {
        await waitForState(page, (t) => t.turning);
        await attachShot(page, testInfo, "bolt-turning: exposed threaded shaft retracting as the hex head turns into its socket");
      }
      await waitForState(page, (t, n) => t.looseBolts.length === n, bolts.length - i - 1);
    }
    const done = await waitForState(page, (s) => s.faults.length === 0);
    expect(done.stats.fixed).toBe(1);
    expect(errors).toEqual([]);
  } finally {
    await attachShot(page, testInfo, 'bolts-fixed: all threaded bolts seated flush against the detailed blue-steel engine assembly');
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
    await attachShot(page, testInfo, 'breach-open: airlock, torn hole in the left wall showing space, white ring of escaping air, engineer beside it holding the tape roll to the hole');
    await page.mouse.down();
    await waitForState(page, (t) => t.faults.length === 0);
    await page.mouse.up();
    const done = await gameState(page);
    expect(done.stats.fixed).toBe(1);
    expect(done.tape).toBeLessThan(18);
    expect(done.entities.tapeStrips).toBeGreaterThan(5);
    expect(errors).toEqual([]);
  } finally {
    await attachShot(page, testInfo, 'breach-sealed: the hole replaced by a woven silver tape patch, layered torn-edge strips on the wall, no air ring');
  }
});

test('tape: pressed right up against the hull, it still goes on the inside', async ({ page }, testInfo) => {
  const errors = collectErrors(page);
  try {
    await openGame(page, { scenario: 'breach' });
    const s = await gameState(page);
    const breach = s.faults[0];
    // Walk straight at the hole until the wall stops the engineer: the roll
    // then reaches past the middle of the wall, nearer its outside face.
    const dx = breach.x - s.player.x;
    const dy = breach.y - s.player.y;
    const key = Math.abs(dx) > Math.abs(dy) ? (dx > 0 ? 'd' : 'a') : dy > 0 ? 'w' : 's';
    await page.keyboard.down(key);
    await waitForState(page, (t, b) => Math.hypot(b.x - t.player.x, b.y - t.player.y) < 15, breach);
    await page.keyboard.up(key);
    await waitForState(page, (t) => !t.player.walking);
    await aimAt(page, breach);
    await page.mouse.down();
    const taping = await waitForState(page, (t) => t.tapeContact !== null && t.strips >= 4);
    await attachShot(page, testInfo, 'breach-flush: airlock, engineer pressed against the left wall, tape feed and woven strips going onto the hole in the wall face, nothing on the floor in front of it');
    // On the hole, on the face the engineer stands at: not the hull's outside.
    const c = taping.tapeContact!;
    expect(Math.hypot(c.x - breach.x, c.y - breach.y)).toBeLessThan(2);
    expect((taping.player.x - c.x) * c.nx + (taping.player.y - c.y) * c.ny).toBeGreaterThan(0);
    const done = await waitForState(page, (t) => t.faults.length === 0);
    await page.mouse.up();
    expect(done.stats.fixed).toBe(1);
    expect(errors).toEqual([]);
  } finally {
    await attachShot(page, testInfo, 'breach-flush-sealed: the hole covered by a woven silver tape patch on the wall face, no strips anywhere else, no air ring');
  }
});

/**
 * Closed-loop steering at the helm: holds the keys that point the marker back
 * at the centre until `done(state)`, then lets go of them.
 */
async function steerToCentre(page: Page, done: (s: BevyState) => boolean | Promise<boolean>) {
  const held = new Set<string>();
  const deadline = Date.now() + 90_000;
  try {
    for (;;) {
      const s = await gameState(page);
      if (await done(s)) return s;
      if (Date.now() > deadline) throw new Error(`steering never finished: ${JSON.stringify(s.nav)}`);
      const want = new Set<string>();
      if (s.nav.x > 0.05) want.add('a');
      if (s.nav.x < -0.05) want.add('d');
      if (s.nav.y > 0.05) want.add('s');
      if (s.nav.y < -0.05) want.add('w');
      for (const k of held) if (!want.has(k)) { await page.keyboard.up(k); held.delete(k); }
      for (const k of want) if (!held.has(k)) { await page.keyboard.down(k); held.add(k); }
      await page.waitForTimeout(100);
    }
  } finally {
    for (const k of held) await page.keyboard.up(k);
  }
}

test('helm: E locks in, WASD steers the marker into the centre band', async ({ page }, testInfo) => {
  const errors = collectErrors(page);
  try {
    await openGame(page, { scenario: 'drift' });
    await waitForState(page, (s) => s.focus === 'Helm' && !s.nav.inBand);
    await attachShot(page, testInfo, 'drift: cockpit, engineer at the pilot seat, nav display below with a ship cursor off-centre on a sweeping radar (amber near the centre square, red further out), display frame blinking red');
    await page.keyboard.press('e');
    await waitForState(page, (s) => s.nav.engaged && s.player.locked);

    let shotTaken = false;
    await steerToCentre(page, async (s) => {
      if (s.faults.length === 0) return true;
      if (!shotTaken && s.nav.inBand && s.faults[0].repair > 0.3) {
        shotTaken = true;
        await attachShot(page, testInfo, 'steering: engineer seated facing the window, ship cursor pale cyan inside the lit centre square, hold bar growing along the display bottom');
      }
      return false;
    });
    const done = await waitForState(page, (s) => !s.nav.engaged);
    expect(done.stats.fixed).toBe(1);
    expect(done.player.locked).toBe(false);
    expect(errors).toEqual([]);
  } finally {
    await attachShot(page, testInfo, 'drift-fixed: marker settling to the centre, display frame steel blue, engineer free to walk');
  }
});

test('helm: leaving mid-hold drains the hold, even with the marker in the band', async ({ page }, testInfo) => {
  const errors = collectErrors(page);
  try {
    await openGame(page, { scenario: 'drift' });
    await waitForState(page, (s) => s.focus === 'Helm');
    await page.keyboard.press('e');
    await waitForState(page, (s) => s.nav.engaged && s.player.locked);
    // Half a hold at the centre, then E: from there the drift needs over 2 s to leave the band.
    await steerToCentre(page, (s) => s.faults.length === 0 || s.faults[0].repair > 0.5);
    await page.keyboard.press('e');
    const left = await waitForState(page, (s) => !s.nav.engaged && !s.player.locked);
    expect(left.faults.length).toBe(1);
    // Only steering from the helm counts.
    await waitForState(
      page,
      (s, r) => s.nav.inBand && s.faults.length === 1 && s.faults[0].repair < r - 0.05,
      left.faults[0].repair,
    );
    await attachShot(page, testInfo, 'helm-left: engineer standing at the pilot seat, ship cursor pale cyan inside the lit centre square, hold bar shrinking along the display bottom');
    const drained = await waitForState(page, (s) => s.faults.length !== 1 || s.faults[0].repair === 0);
    expect(drained.faults.length).toBe(1);
    expect(drained.stats.fixed).toBe(0);
    expect(errors).toEqual([]);
  } finally {
    await attachShot(page, testInfo, 'helm-left-drained: engineer standing at the pilot seat, no hold bar on the nav display, the drift still active');
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
    await attachShot(page, testInfo, 'diagnostics: minimap of the five rooms stacked top to bottom, three red markers on the left engine, one on the airlock left wall, cyan dot for the engineer in the quarters, list of two faults with seconds left');
    await page.keyboard.press('e');
    await waitForState(page, (s) => s.diag === 'Closed');
    expect(errors).toEqual([]);
  } finally {
    await attachShot(page, testInfo, 'diagnostics-closed: quarters again, the console screen dark, no minimap, faint red alarm tint');
  }
});
