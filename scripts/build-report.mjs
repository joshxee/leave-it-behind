#!/usr/bin/env node
// Builds <dir>/REPORT.md from <dir>/summary.json, <dir>/logs/test.log (cargo
// test output) and <dir>/playwright-results.json. Plain Node, no
// dependencies, deterministic: the same inputs always give the same report.
// Nothing in the report is written by a model.
//
//   node scripts/build-report.mjs test-reports/<ts>
//
// Exits 1 when the report status is FAIL.
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { join, relative, resolve, sep } from 'node:path';

const dir = resolve(process.argv[2] ?? 'test-reports/latest');
const MAX_LINES = 30;
const read = (p) => (existsSync(p) ? readFileSync(p, 'utf8') : null);
const stripAnsi = (s) => (s ?? '').replace(/\u001b\[[0-9;]*[A-Za-z]/g, '');
const trim = (text) => {
  const lines = stripAnsi(text).split('\n').map((l) => l.replace(/\s+$/, '')).filter((l, i, a) => l || (i > 0 && a[i - 1]));
  return lines.length > MAX_LINES ? [...lines.slice(0, MAX_LINES), `… (${lines.length - MAX_LINES} more lines)`] : lines;
};
const rel = (p) => relative(dir, p).split(sep).join('/');
const fmtSecs = (s) => (s == null ? '' : `${Number(s).toFixed(1)}s`);

const summaryText = read(join(dir, 'summary.json'));
if (!summaryText) {
  console.error(`no summary.json in ${dir}`);
  process.exit(1);
}
const summary = JSON.parse(summaryText);
const steps = Object.fromEntries(summary.steps.map((s) => [s.step, s]));

// ---- cargo test (libtest text output) --------------------------------------
function parseCargoTest(log) {
  const kinds = { unit: { pass: 0, fail: 0, skip: 0, secs: 0 }, integration: { pass: 0, fail: 0, skip: 0, secs: 0 } };
  const failures = [];
  if (!log) return { kinds, failures, ran: false };
  let kind = 'unit';
  let ran = false;
  const lines = stripAnsi(log).split('\n');
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    const running = line.match(/^\s+Running (\S+)/);
    if (running) kind = running[1].startsWith('tests') ? 'integration' : 'unit';
    if (/^\s+Doc-tests /.test(line)) kind = 'unit';
    const t = line.match(/^test (\S+) \.\.\. (ok|FAILED|ignored)/);
    if (t) {
      ran = true;
      kinds[kind][t[2] === 'ok' ? 'pass' : t[2] === 'FAILED' ? 'fail' : 'skip']++;
    }
    const fin = line.match(/^test result: .*finished in ([\d.]+)s/);
    if (fin) kinds[kind].secs += Number(fin[1]);
    const out = line.match(/^---- (\S+) stdout ----$/);
    if (out) {
      const body = [];
      for (let j = i + 1; j < lines.length && !/^---- \S+ stdout ----$/.test(lines[j]) && !/^failures:$/.test(lines[j]); j++) body.push(lines[j]);
      const panic = body.join('\n').match(/panicked at ([^\n]+?):(\d+):\d+:/);
      failures.push({
        kind,
        name: out[1],
        location: panic ? `${panic[1]}:${panic[2]}` : '(unknown)',
        // Stop at the backtrace: the panic line and assertion are what matter.
        message: trim(body.slice(0, body.findIndex((l) => /^stack backtrace:/.test(l)) >>> 0).filter((l) => !/^note: (run with `RUST_BACKTRACE|Some details are omitted)/.test(l)).join('\n')),
      });
    }
  }
  return { kinds, failures, ran };
}

// ---- Playwright JSON --------------------------------------------------------
function parsePlaywright(json) {
  const res = { pass: 0, fail: 0, skip: 0, flaky: 0, secs: 0, failures: [], flakyTests: [], shots: [] };
  if (!json) return res;
  const data = JSON.parse(json);
  const e2eDir = join(dir, 'screenshots', 'e2e');
  // Attachment paths point into e2e/test-results; test-all.sh copies that tree to screenshots/e2e.
  const mapPath = (p) => {
    const m = p.split(/[\\/]test-results[\\/]/);
    return m.length > 1 ? rel(join(e2eDir, m[1])) : p;
  };
  const walk = (suite, titles) => {
    const t = suite.title && !suite.title.endsWith('.ts') ? [...titles, suite.title] : titles;
    for (const spec of suite.specs ?? []) {
      for (const test of spec.tests ?? []) {
        const runs = test.results ?? [];
        const name = [...t, spec.title].join(' › ');
        const where = `e2e/specs/${spec.file}:${spec.line}`;
        res.secs += runs.reduce((a, r) => a + (r.duration ?? 0), 0) / 1000;
        const bucket = { expected: 'pass', unexpected: 'fail', flaky: 'flaky', skipped: 'skip' }[test.status] ?? 'fail';
        res[bucket]++;
        const last = runs.at(-1) ?? {};
        const atts = runs.flatMap((r) => r.attachments ?? []).filter((a) => a.path);
        for (const a of atts.filter((a) => a.contentType === 'image/png')) {
          res.shots.push({
            test: name,
            path: mapPath(a.path),
            description: a.name === 'screenshot' ? '(automatic end-of-test page screenshot)' : a.name,
          });
        }
        if (bucket === 'fail') {
          const errs = (last.errors?.length ? last.errors : [last.error]).filter(Boolean);
          res.failures.push({
            name,
            location: errs[0]?.location ? `${relative(process.cwd(), errs[0].location.file).split(sep).join('/')}:${errs[0].location.line}` : where,
            message: trim(errs.map((e) => e.message ?? String(e)).join('\n')),
            artifacts: atts.map((a) => `${a.name}: ${mapPath(a.path)}`),
          });
        }
        if (bucket === 'flaky') res.flakyTests.push({ name, where, retries: runs.length - 1 });
      }
    }
    for (const child of suite.suites ?? []) walk(child, t);
  };
  for (const s of data.suites ?? []) walk(s, []);
  return res;
}

const cargo = parseCargoTest(read(join(dir, 'logs', 'test.log')));
const pw = parsePlaywright(read(join(dir, 'playwright-results.json')));

// ---- summary table ----------------------------------------------------------
const stepStatus = (name) => steps[name]?.status ?? 'not run';
const rows = [];
const row = (layer, status, pass, fail, skip, secs) => rows.push(`| ${layer} | ${status} | ${pass} | ${fail} | ${skip} | ${secs} |`);
if (steps.fmt) row('fmt (+ CLAUDE.md ≤150 lines)', stepStatus('fmt'), '', '', '', fmtSecs(steps.fmt.duration_s));
if (steps.clippy) row('clippy', stepStatus('clippy'), '', '', '', fmtSecs(steps.clippy.duration_s));
if (steps.test) {
  const testStatus = (k) => (!cargo.ran ? steps.test.status : cargo.kinds[k].fail ? 'fail' : 'pass');
  row('unit', testStatus('unit'), cargo.kinds.unit.pass, cargo.kinds.unit.fail, cargo.kinds.unit.skip, fmtSecs(cargo.kinds.unit.secs));
  row('integration', testStatus('integration'), cargo.kinds.integration.pass, cargo.kinds.integration.fail, cargo.kinds.integration.skip, fmtSecs(cargo.kinds.integration.secs));
}
if (steps.smoke) row('native smoke', stepStatus('smoke'), '', '', '', fmtSecs(steps.smoke.duration_s));
if (steps['web-build']) row('web build', stepStatus('web-build'), '', '', '', fmtSecs(steps['web-build'].duration_s));
if (steps.e2e) row('e2e (Playwright)', stepStatus('e2e'), pw.pass + pw.flaky, pw.fail, pw.skip, fmtSecs(pw.secs || steps.e2e.duration_s));

const failed = summary.steps.some((s) => s.status === 'fail') || cargo.failures.length > 0 || pw.fail > 0;
const out = [
  `# Test report ${summary.timestamp} (scope: ${summary.scope})`,
  '',
  '| Layer | Status | Pass | Fail | Skip | Duration |',
  '|---|---|---|---|---|---|',
  ...rows,
  '',
];

// Steps that failed without individual test failures (compile errors, fmt, clippy, smoke, build).
const explained = new Set();
if (cargo.failures.length) explained.add('test');
if (pw.failures.length) explained.add('e2e');
const stepFailures = summary.steps.filter((s) => s.status === 'fail' && !explained.has(s.step));

if (failed) {
  out.push('## Failures', '');
  for (const f of cargo.failures) {
    out.push(`### ${f.kind}: \`${f.name}\``, '', `- Location: \`${f.location}\``, `- Log: \`logs/test.log\``, '', '```', ...f.message, '```', '');
  }
  for (const f of pw.failures) {
    out.push(`### e2e: ${f.name}`, '', `- Location: \`${f.location}\``, ...f.artifacts.map((a) => `- ${a}`), '', '```', ...f.message, '```', '');
  }
  for (const s of stepFailures) {
    const log = read(join(dir, s.log)) ?? '';
    const errorLines = stripAnsi(log).split('\n').filter((l) => /error|panicked|warning: |Diff in|FAILED|exceeds|leak check|not found/i.test(l));
    out.push(`### step: ${s.step} (exit ${s.exit_code})`, '', `- Log: \`${s.log}\``, '', '```', ...trim((errorLines.length ? errorLines : log.split('\n').slice(-MAX_LINES)).join('\n')), '```', '');
    if (s.step === 'smoke' && existsSync(join(dir, 'screenshots', 'smoke-native.png'))) out.push('- Screenshot: `screenshots/smoke-native.png`', '');
  }
}

if (pw.flakyTests.length) {
  out.push('## ⚠️ Flaky (passed only on retry)', '');
  for (const f of pw.flakyTests) out.push(`- ${f.name} (\`${f.where}\`, ${f.retries} retr${f.retries === 1 ? 'y' : 'ies'})`);
  out.push('');
}

const shots = [...pw.shots];
if (existsSync(join(dir, 'screenshots', 'smoke-native.png'))) {
  shots.unshift({ test: 'native smoke (ci/smoke.ron)', path: 'screenshots/smoke-native.png', description: "level one at frame 60: engineer's quarters, orange engineer holding the wrench, ARRIVAL IN countdown top centre, tool belt bottom, version bottom-right" });
}
out.push('## Screenshots index', '');
if (!shots.length) out.push('(none)');
for (const s of shots) out.push(`- \`${s.path}\` (${s.test}): ${s.description}`);
out.push('', '## Triage', '', '', `STATUS: ${failed ? 'FAIL' : 'PASS'}`);

writeFileSync(join(dir, 'REPORT.md'), out.join('\n') + '\n');
console.log(`${rel(join(dir, 'REPORT.md'))}: STATUS: ${failed ? 'FAIL' : 'PASS'}`);
process.exit(failed ? 1 : 0);
