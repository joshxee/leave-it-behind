#!/usr/bin/env node
// Drives the e2e web build and records what happened. Used by the run-game
// and verify skills. Needs `E2E=1 scripts/build-web.sh` and a server on PORT
// (default 4173; scripts/serve-web.sh). Starts the server itself if none.
//
//   node e2e/tools/capture.mjs --out test-reports/verify-<ts>
//       [--scenario bolts]             start from a scenario fixture
//       [--keys "KeyD:500,Digit2,Move:640:200,Click,Click:3000"]
//                                       one step per entry: KEY presses a key,
//                                       KEY:ms holds it; Move:X:Y moves the
//                                       mouse to canvas pixel (X, Y); Click
//                                       clicks, Click:ms holds the left button;
//                                       Wheel:DY scrolls
//       [--wait-state '<js predicate>'] after the inputs, wait until the
//                                       predicate over `s` (= __bevyState) is true
//       [--freeze --ticks 30]           deterministic: step N fixed ticks per step
//
// Writes <out>/NN-<step>.png after each step, <out>/states.json (state per
// step), <out>/console.json, and <out>/result.json. Exits 1 on console
// errors, page errors, a crash overlay, or __bevyReady never becoming true.
import { chromium } from '@playwright/test';
import { spawn } from 'node:child_process';
import { mkdirSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const root = resolve(here, '..', '..');
const argv = process.argv.slice(2);
const opt = (name, fallback) => {
  const i = argv.indexOf(`--${name}`);
  return i >= 0 && argv[i + 1] && !argv[i + 1].startsWith('--') ? argv[i + 1] : fallback;
};
const flag = (name) => argv.includes(`--${name}`);

const ts = new Date().toISOString().replace(/[:.]/g, '-');
const out = resolve(opt('out', join(root, 'test-reports', `verify-${ts}`)));
const scenario = opt('scenario');
const keys = (opt('keys', '') || '').split(',').map((k) => k.trim()).filter(Boolean);
const waitState = opt('wait-state');
const freeze = flag('freeze');
const ticks = Number(opt('ticks', 30));
const port = Number(process.env.PORT ?? 4173);
const base = `http://127.0.0.1:${port}`;

const up = async () => {
  try {
    return (await fetch(`${base}/index.html`)).ok;
  } catch {
    return false;
  }
};

let server;
if (!(await up())) {
  server = spawn('bash', [join(root, 'scripts', 'serve-web.sh')], { env: { ...process.env, PORT: String(port) }, stdio: 'ignore' });
  for (let i = 0; i < 60 && !(await up()); i++) await new Promise((r) => setTimeout(r, 500));
}

mkdirSync(out, { recursive: true });
const consoleLog = [];
const errors = [];
const states = [];
const shots = [];
let ok = true;
let reason = '';

const browser = await chromium.launch({
  args: ['--use-angle=swiftshader', '--enable-unsafe-swiftshader', '--ignore-gpu-blocklist'],
});
try {
  const page = await browser.newPage({ viewport: { width: 1280, height: 720 } });
  page.on('console', (m) => {
    consoleLog.push({ type: m.type(), text: m.text() });
    if (m.type() === 'error' && !m.text().includes('AudioContext was not allowed')) errors.push(m.text());
  });
  page.on('pageerror', (e) => {
    if (!e.message.includes("Using exceptions for control flow")) errors.push(`pageerror: ${e.message}`);
  });

  const params = new URLSearchParams();
  if (scenario) params.set('scenario', scenario);
  if (freeze) params.set('freeze', '1');
  await page.goto(`${base}/index.html?${params}`);

  const shot = async (label) => {
    const file = join(out, `${String(shots.length).padStart(2, '0')}-${label.replace(/[^\w-]+/g, '_')}.png`);
    await page.locator('#bevy-canvas').screenshot({ path: file });
    const state = await page.evaluate(() => window.__bevyState ?? null);
    shots.push(file);
    states.push({ step: label, screenshot: file, state });
  };

  try {
    await page.waitForFunction(() => window.__bevyReady === true, null, { timeout: 90_000 });
  } catch {
    ok = false;
    reason = '__bevyReady never became true';
    await page.screenshot({ path: join(out, '00-not-ready.png') });
    throw new Error(reason);
  }
  await page.locator('#bevy-canvas').focus();
  const advance = async () => {
    if (freeze) {
      const t = await page.evaluate(() => window.__bevyState.tick);
      await page.evaluate((n) => window.__bevyStep(n), ticks);
      await page.waitForFunction((x) => window.__bevyState.tick >= x && window.__bevyState.frozen, t + ticks, { timeout: 60_000 });
    } else {
      await page.waitForTimeout(300);
    }
  };

  await advance();
  await shot('initial');
  for (const spec of keys) {
    const [key, hold, extra] = spec.split(':');
    if (key === 'Move') {
      await page.mouse.move(Number(hold), Number(extra));
      await advance();
    } else if (key === 'Wheel') {
      await page.mouse.wheel(0, Number(hold));
      await advance();
    } else if (key === 'Click' && hold) {
      await page.mouse.down();
      if (freeze) await advance();
      else await page.waitForTimeout(Number(hold));
      await page.mouse.up();
    } else if (key === 'Click') {
      await page.mouse.down();
      await page.mouse.up();
      await advance();
    } else if (hold) {
      await page.keyboard.down(key);
      if (freeze) await advance();
      else await page.waitForTimeout(Number(hold));
      await page.keyboard.up(key);
    } else {
      await page.keyboard.press(key);
      await advance();
    }
    await shot(spec);
  }
  if (waitState) {
    try {
      await page.waitForFunction(`(() => { const s = window.__bevyState; return !!(s && (${waitState})); })()`, null, { timeout: 30_000 });
      await shot('wait-state');
    } catch {
      ok = false;
      reason = `wait-state never true: ${waitState}`;
      await shot('wait-state-timeout');
    }
  }
  if (await page.locator('#crash-overlay').isVisible()) {
    ok = false;
    reason = 'crash overlay visible';
  }
} catch (e) {
  ok = false;
  reason ||= String(e.message ?? e);
} finally {
  await browser.close();
  server?.kill();
}

if (errors.length) {
  ok = false;
  reason ||= `${errors.length} console error(s)`;
}
writeFileSync(join(out, 'states.json'), JSON.stringify(states, null, 2));
writeFileSync(join(out, 'console.json'), JSON.stringify(consoleLog, null, 2));
const result = { ok, reason, scenario: scenario ?? 'default', keys, freeze, errors, screenshots: shots, final: states.at(-1)?.state ?? null };
writeFileSync(join(out, 'result.json'), JSON.stringify(result, null, 2));
console.log(JSON.stringify(result, null, 2));
process.exit(ok ? 0 : 1);
