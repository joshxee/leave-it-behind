---
name: verify
description: Confirm a code change works in the running game (browser build driven by keyboard, screenshots read and compared), without falling back to tests or type checks. Project override of the built-in /verify.
allowed-tools: Bash(E2E=1 scripts/build-web.sh) Bash(scripts/serve-web.sh) Bash(node e2e/tools/capture.mjs *) Bash(scripts/smoke-native.sh) Bash(curl -sf http://localhost:4173/index.html) Bash(pkill -f "[h]ttp-server") Bash(pkill -f "[h]ttp.server 4173")
---

# Verify a change in the running game

Follow `.claude/skills/run-game/SKILL.md` for every build, launch, drive, and
teardown command. Don't restate or improvise them.

1. **Launch and drive.** Build the e2e web version and run `capture.mjs` with
   inputs that exercise the change. Use `--scenario` to jump to the affected
   situation, `--keys` for the interaction, and `--wait-state` for the expected
   state. Use `--freeze --ticks N` when you need exact tick counts.
2. **Look.** Open every screenshot `capture.mjs` wrote with the Read tool.
   Compare each with what the change should look like, and report what you
   actually saw: positions, text, colors, and anything unexpected. Check
   `result.json`. Console errors, Rust panics, the crash overlay, or
   `__bevyReady` never becoming true all mean **failure**, whatever the screenshots show.
3. **Native-only changes** (window, audio device, desktop-specific code): run
   `scripts/smoke-native.sh` instead and Read `test-reports/smoke/smoke.png`.
4. Do not offer unit tests, clippy, or type checks as proof. That is `/test-report`'s job.
5. **Tear down** the server as `run-game` describes.

Report: what you ran (scenario, keys), what each screenshot showed, the final
`__bevyState`, and a verdict: works / does not work / could not verify (why).

The Playwright MCP server, if configured, may be used for exploratory
clicking, but `capture.mjs` is the reproducible path, and its output is what you report.

Edit this file only when it sent a run wrong (a command that failed, a
missing step). Never record per-session notes here.
