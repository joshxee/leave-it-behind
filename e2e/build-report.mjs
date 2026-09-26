#!/usr/bin/env node
// Turns e2e/results.json (Playwright JSON reporter) into
// test-reports/e2e/summary.md, listing failures, flaky tests (passed only on
// retry, shown as warnings), and every attached screenshot.
//
//   node e2e/build-report.mjs [results.json] [out-dir]
import { readFileSync, mkdirSync, writeFileSync, existsSync } from 'node:fs';
import { dirname, join, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const input = resolve(process.argv[2] ?? join(here, 'results.json'));
const outDir = resolve(process.argv[3] ?? join(here, '..', 'test-reports', 'e2e'));

if (!existsSync(input)) {
  console.error(`no results at ${input}; run the e2e tests first`);
  process.exit(1);
}
const results = JSON.parse(readFileSync(input, 'utf8'));

const rows = [];
function walk(suite, prefix = []) {
  const title = suite.title ? [...prefix, suite.title] : prefix;
  for (const spec of suite.specs ?? []) {
    for (const test of spec.tests ?? []) {
      const runs = test.results ?? [];
      const last = runs.at(-1);
      rows.push({
        name: [...title, spec.title].filter(Boolean).join(' › '),
        outcome: test.status, // expected | unexpected | flaky | skipped
        retries: Math.max(0, runs.length - 1),
        error: runs.map((r) => r.error?.message).filter(Boolean).at(-1),
        attachments: runs.flatMap((r) => (r.attachments ?? []).filter((a) => a.path && a.contentType === 'image/png')),
        duration: runs.reduce((t, r) => t + (r.duration ?? 0), 0),
        lastStatus: last?.status,
      });
    }
  }
  for (const child of suite.suites ?? []) walk(child, title);
}
for (const suite of results.suites ?? []) walk(suite);

const by = (o) => rows.filter((r) => r.outcome === o);
const failed = by('unexpected');
const flaky = by('flaky');
const passed = by('expected');
const skipped = by('skipped');

const strip = (s) => (s ?? '').replace(/\u001b\[[0-9;]*m/g, '');
const lines = [
  '# E2E report',
  '',
  `**${passed.length} passed, ${failed.length} failed, ${flaky.length} flaky, ${skipped.length} skipped**`,
  '',
];
if (failed.length) {
  lines.push('## Failed', '');
  for (const r of failed) lines.push(`- ❌ ${r.name}`, '', '```', strip(r.error).slice(0, 2000), '```', '');
}
if (flaky.length) {
  lines.push('## ⚠️ Flaky (passed only on retry)', '');
  for (const r of flaky) lines.push(`- ⚠️ ${r.name} (${r.retries} retr${r.retries === 1 ? 'y' : 'ies'})`);
  lines.push('');
}
if (passed.length) {
  lines.push('## Passed', '');
  for (const r of passed) lines.push(`- ✅ ${r.name} (${(r.duration / 1000).toFixed(1)}s)`);
  lines.push('');
}
if (skipped.length) {
  lines.push('## Skipped', '');
  for (const r of skipped) lines.push(`- ${r.name}`);
  lines.push('');
}
lines.push('## Screenshots', '');
for (const r of rows) {
  for (const a of r.attachments) lines.push(`- ${r.name}: \`${a.name}\` → ${relative(process.cwd(), a.path)}`);
}

mkdirSync(outDir, { recursive: true });
writeFileSync(join(outDir, 'summary.md'), lines.join('\n') + '\n');
writeFileSync(
  join(outDir, 'summary.json'),
  JSON.stringify({ passed: passed.length, failed: failed.map((r) => r.name), flaky: flaky.map((r) => r.name), skipped: skipped.length }, null, 2),
);
console.log(lines.slice(0, 3).join('\n'));
if (flaky.length) console.warn(`warning: ${flaky.length} flaky test(s): ${flaky.map((r) => r.name).join(', ')}`);
console.log(`report: ${relative(process.cwd(), join(outDir, 'summary.md'))}`);
if (process.env.GITHUB_STEP_SUMMARY) writeFileSync(process.env.GITHUB_STEP_SUMMARY, lines.join('\n') + '\n', { flag: 'a' });
