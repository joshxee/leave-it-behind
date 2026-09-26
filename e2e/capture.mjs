#!/usr/bin/env node
// Captures a screenshot of the e2e web build at an exact tick, optionally
// starting from a scenario fixture. Used by the run-game and verify skills.
//
//   node e2e/capture.mjs [--scenario score_nine] [--ticks 30] [--keys Space,ArrowRight]
//                        [--out test-reports/captures/<scenario>.png]
//
// Needs `E2E=1 scripts/build-web.sh` first. Starts serve-web.sh if nothing is
// listening on PORT (default 4173). Prints the final __bevyState as JSON.
import { chromium } from '@playwright/test';
import { spawn } from 'node:child_process';
import { mkdirSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const args = Object.fromEntries(
  process.argv.slice(2).reduce((acc, a, i, all) => (a.startsWith('--') ? [...acc, [a.slice(2), all[i + 1]]] : acc), []),
);
const scenario = args.scenario ?? 'default';
const ticks = Number(args.ticks ?? 30);
const keys = (args.keys ?? '').split(',').filter(Boolean);
const port = Number(process.env.PORT ?? 4173);
const out = resolve(args.out ?? join(here, '..', 'test-reports', 'captures', `${scenario}.png`));
const base = `http://127.0.0.1:${port}`;

async function up() {
  try {
    return (await fetch(`${base}/index.html`)).ok;
  } catch {
    return false;
  }
}

let server;
if (!(await up())) {
  server = spawn('bash', [join(here, '..', 'scripts', 'serve-web.sh')], { env: { ...process.env, PORT: String(port) }, stdio: 'ignore' });
  for (let i = 0; i < 60 && !(await up()); i++) await new Promise((r) => setTimeout(r, 500));
}

const browser = await chromium.launch({ args: ['--use-angle=swiftshader', '--enable-unsafe-swiftshader', '--ignore-gpu-blocklist'] });
try {
  const page = await browser.newPage({ viewport: { width: 1280, height: 720 } });
  page.on('console', (m) => m.type() === 'error' && console.error(`[console] ${m.text()}`));
  await page.goto(`${base}/index.html?scenario=${encodeURIComponent(scenario)}&freeze=1`);
  await page.waitForFunction(() => window.__bevyReady === true, null, { timeout: 90_000 });
  await page.locator('#bevy-canvas').focus();
  for (const key of keys) await page.keyboard.press(key);
  const start = await page.evaluate(() => window.__bevyState.tick);
  await page.evaluate((n) => window.__bevyStep(n), ticks);
  await page.waitForFunction((t) => window.__bevyState.tick >= t && window.__bevyState.frozen, start + ticks, { timeout: 60_000 });
  mkdirSync(dirname(out), { recursive: true });
  await page.locator('#bevy-canvas').screenshot({ path: out });
  console.log(JSON.stringify(await page.evaluate(() => window.__bevyState)));
  console.log(`screenshot: ${out}`);
} finally {
  await browser.close();
  server?.kill();
}
