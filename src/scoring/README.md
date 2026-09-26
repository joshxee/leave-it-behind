# scoring

+1 point per Space press. Reaching `SCORE_THRESHOLD` (10) sends `ThresholdReached` once.

- **Resources/messages:** `Score`, `ScoreChanged { score }`, `ThresholdReached`.
- **Systems:** `apply_score_presses` (FixedUpdate/Resolve) drains `PlayerIntent::score_presses`.
- **Pure functions:** `add_points`.
- **Tests:** unit tests in `mod.rs`; integration tests in `tests/integration/scoring.rs` and `tests/integration/scenarios.rs`; e2e in `e2e/specs/interaction.spec.ts`.
- **Scenarios:** `default`, `score_nine` (one below the threshold).
