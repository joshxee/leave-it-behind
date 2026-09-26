import { expect, type Page, type TestInfo } from '@playwright/test';

export type BevyState = {
  state: string;
  tick: number;
  score: number;
  frozen: boolean;
  ready: boolean;
  player: { x: number; y: number };
  entities: { players: number };
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

/** Opens the game and waits for the e2e bridge to report ready. */
export async function openGame(page: Page, params: Record<string, string> = {}) {
  const query = new URLSearchParams(params).toString();
  await page.goto(`/index.html${query ? `?${query}` : ''}`);
  await page.waitForFunction(() => window.__bevyReady === true, null, { timeout: 90_000 });
  await page.waitForFunction(() => window.__bevyState?.ready === true);
}

export async function gameState(page: Page): Promise<BevyState> {
  const state = await page.evaluate(() => window.__bevyState);
  expect(state, '__bevyState should be published').toBeTruthy();
  return state!;
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
export async function attachShot(page: Page, testInfo: TestInfo, name: string, canvasOnly = false) {
  const body = canvasOnly
    ? await page.locator('#bevy-canvas').screenshot()
    : await page.screenshot({ fullPage: true });
  await testInfo.attach(name, { body, contentType: 'image/png' });
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
