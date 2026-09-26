import { expect, type Page, type TestInfo } from '@playwright/test';

export type FaultState = {
  kind: 'LooseBolts' | 'HullBreach' | 'TrajectoryDrift';
  site: string;
  room: string;
  remaining: number;
  repair: number;
  x: number;
  y: number;
};

/** `window.__bevyState`, published by `src/e2e_bridge.rs`. World units; y up. */
export type BevyState = {
  state: 'Boot' | 'Playing' | 'Landed' | 'Lost';
  tick: number;
  frozen: boolean;
  ready: boolean;
  room: string;
  /** Center of the current room: the camera's (unshaken) position. */
  camera: { x: number; y: number };
  player: { x: number; y: number; fx: number; fy: number; locked: boolean };
  focus: 'Helm' | 'Diagnostics' | null;
  tool: 'Wrench' | 'Tape';
  tape: number;
  snap: boolean;
  turning: boolean;
  taping: boolean;
  strips: number;
  journey: { elapsed: number; remaining: number; duration: number };
  faults: FaultState[];
  looseBolts: { x: number; y: number }[];
  nav: { engaged: boolean; x: number; y: number; inBand: boolean };
  diag: 'Closed' | 'Scanning' | 'Open';
  diagUses: number;
  alarm: { level: number; active: number; jolt: number };
  stats: { started: number; fixed: number; failure: FaultState['kind'] | null };
  entities: { players: number; faults: number; tapeStrips: number };
};

declare global {
  interface Window {
    __bevyReady?: boolean;
    __bevyState?: BevyState;
    __bevyStep?: (n: number) => void;
  }
}

// Errors the browser or wasm-bindgen emit that are not game failures.
const IGNORED = [
  "Using exceptions for control flow, don't mind me",
  // Chrome's autoplay policy until the first user gesture.
  'The AudioContext was not allowed to start',
];

/** Collects console errors and page errors. Call before `page.goto`. */
export function collectErrors(page: Page): string[] {
  const errors: string[] = [];
  page.on('console', (msg) => {
    if (msg.type() === 'error' && !IGNORED.some((s) => msg.text().includes(s))) {
      errors.push(`console: ${msg.text()} (${msg.location().url})`);
    }
  });
  page.on('pageerror', (err) => {
    if (!IGNORED.some((s) => err.message.includes(s))) errors.push(`pageerror: ${err.message}`);
  });
  return errors;
}

/**
 * Opens the game, waits for the e2e bridge to report ready, and focuses the
 * canvas so keyboard input reaches the game (focus, not click: a click would
 * also use the held tool).
 */
export async function openGame(page: Page, params: Record<string, string> = {}) {
  const query = new URLSearchParams(params).toString();
  await page.goto(`/index.html${query ? `?${query}` : ''}`);
  await page.waitForFunction(() => window.__bevyReady === true, null, { timeout: 90_000 });
  await page.waitForFunction(() => window.__bevyState?.ready === true);
  await page.locator('#bevy-canvas').focus();
}

export async function gameState(page: Page): Promise<BevyState> {
  const state = await page.evaluate(() => window.__bevyState);
  expect(state, '__bevyState should be published').toBeTruthy();
  return state!;
}

/**
 * Waits until `predicate(state, arg)` holds, polling inside the page (one
 * traced action, not hundreds), then returns the current state. The
 * predicate runs in the browser: it cannot capture variables, so pass them
 * as `arg`. Game time is one tick per rendered frame, so allow for slow
 * software rendering in `timeout`.
 */
export async function waitForState<A = undefined>(
  page: Page,
  predicate: (s: BevyState, arg: A) => boolean,
  arg?: A,
  timeout = 60_000,
): Promise<BevyState> {
  try {
    await page.waitForFunction(
      ({ src, arg }) => {
        const s = window.__bevyState;
        const pred = new Function(`return (${src});`)() as (s: unknown, a: unknown) => boolean;
        return !!s && pred(s, arg);
      },
      { src: predicate.toString(), arg },
      { timeout, polling: 50 },
    );
  } catch {
    const last = await page.evaluate(() => window.__bevyState);
    throw new Error(`state never matched: ${predicate} (arg ${JSON.stringify(arg)})\nlast: ${JSON.stringify(last)}`);
  }
  return gameState(page);
}

/** Page pixel under world point (x, y): the canvas is 1280×720 at the page origin. */
export function toScreen(s: BevyState, p: { x: number; y: number }) {
  return { x: 640 + (p.x - s.camera.x), y: 360 - (p.y - s.camera.y) };
}

/** Moves the mouse over world point `p`. */
export async function aimAt(page: Page, p: { x: number; y: number }) {
  const s = await gameState(page);
  const screen = toScreen(s, p);
  await page.mouse.move(screen.x, screen.y);
}

/** Advances exactly `n` fixed ticks (requires `?freeze=1`). */
export async function step(page: Page, n: number) {
  const target = (await gameState(page)).tick + n;
  await page.evaluate((k) => window.__bevyStep!(k), n);
  await page.waitForFunction((t) => (window.__bevyState?.tick ?? 0) >= t && window.__bevyState?.frozen, target);
}

/**
 * Attaches a screenshot to the report, pass or fail. `name` is
 * "<label>: <what the screenshot should show>"; the report's screenshot
 * index uses it verbatim.
 */
export async function attachShot(page: Page, testInfo: TestInfo, name: string, canvasOnly = true) {
  // Written to a file (not attached as a body) so the JSON results carry a path for REPORT.md.
  const label = name.split(':')[0].trim().replace(/[^\w-]+/g, '_');
  const path = testInfo.outputPath(`${label}.png`);
  if (canvasOnly) await page.locator('#bevy-canvas').screenshot({ path });
  else await page.screenshot({ path, fullPage: true });
  await testInfo.attach(name, { path, contentType: 'image/png' });
}

/**
 * Counts distinct colors in a PNG (sampled on a grid), decoding it in the
 * browser so no image library is needed. Stops counting at `limit`.
 */
export async function distinctColors(page: Page, png: Buffer, limit = 16): Promise<number> {
  return page.evaluate(
    async ({ b64, limit }) => {
      const blob = await (await fetch(`data:image/png;base64,${b64}`)).blob();
      const bmp = await createImageBitmap(blob);
      const c = new OffscreenCanvas(bmp.width, bmp.height);
      const ctx = c.getContext('2d')!;
      ctx.drawImage(bmp, 0, 0);
      const { data } = ctx.getImageData(0, 0, bmp.width, bmp.height);
      const seen = new Set<number>();
      for (let y = 0; y < bmp.height; y += 4) {
        for (let x = 0; x < bmp.width; x += 4) {
          const i = (y * bmp.width + x) * 4;
          seen.add((data[i] << 16) | (data[i + 1] << 8) | data[i + 2]);
          if (seen.size >= limit) return seen.size;
        }
      }
      return seen.size;
    },
    { b64: png.toString('base64'), limit },
  );
}

/**
 * Average color of a canvas region, sampled in the page. Used to check tints
 * without pixel baselines.
 */
export async function averageColor(
  page: Page,
  png: Buffer,
  region: { x: number; y: number; w: number; h: number },
): Promise<{ r: number; g: number; b: number }> {
  return page.evaluate(
    async ({ b64, region }) => {
      const blob = await (await fetch(`data:image/png;base64,${b64}`)).blob();
      const bmp = await createImageBitmap(blob);
      const c = new OffscreenCanvas(bmp.width, bmp.height);
      const ctx = c.getContext('2d')!;
      ctx.drawImage(bmp, 0, 0);
      const { data } = ctx.getImageData(region.x, region.y, region.w, region.h);
      let r = 0, g = 0, b = 0;
      const n = data.length / 4;
      for (let i = 0; i < data.length; i += 4) {
        r += data[i];
        g += data[i + 1];
        b += data[i + 2];
      }
      return { r: r / n, g: g / n, b: b / n };
    },
    { b64: png.toString('base64'), region },
  );
}
