---
name: test-report
description: Run the full test suite (fmt, clippy, cargo test, native smoke, web build, Playwright) on a cheap model and return only the REPORT.md path and STATUS line.
argument-hint: "[--scope all|rust|native|web]"
context: fork
agent: test-runner
model: haiku
background: false
---

Follow your test-runner agent instructions exactly. Run `scripts/test-all.sh $ARGUMENTS`.
Return only two lines: the `REPORT.md` path and the `STATUS:` line.
