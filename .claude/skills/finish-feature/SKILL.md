---
name: finish-feature
description: End-of-feature workflow. Check tests exist at the right layers, /verify visible changes, run /test-report, fix failures (max 3 loops), and summarize.
---

# Finish a feature

1. **Tests.** Check the feature has tests at the right layers (see
   `TESTING.md`): unit tests for pure logic, ECS integration tests in
   `tests/integration/<feature>.rs`, a scenario in `src/scenarios/` for any new
   game situation, and a Playwright spec for anything visible or interactive.
   Add what's missing. Make sure the feature's `src/<feature>/README.md` lists its scenarios and tests.
2. **Verify.** For any visible or interactive change, run `/verify` and read its screenshots.
3. **Test report.** Run `/test-report`. It runs the full suite on a cheap model and returns a `REPORT.md` path and a STATUS line.
4. **Read `REPORT.md`.** On `STATUS: FAIL`, you (the primary model, not the
   test runner) open the referenced screenshots and traces, diagnose the
   failure, fix it, and go back to step 3. The Triage lines are low-confidence
   hints only. After **3** failing loops, stop and ask the user, with the report path and what you tried.
   Never "fix" by raising `ci/budgets.env`, updating snapshot baselines, or weakening a test without asking.
5. **On `STATUS: PASS`**, summarize: the feature, the tests added (per layer),
   what `/verify` saw, and the `REPORT.md` path.
