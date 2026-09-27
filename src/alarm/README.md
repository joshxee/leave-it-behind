# alarm

Ambient cues: they say something is wrong and how urgent it is, never what or where (the HUD's vitals panel says what, the diagnostic screen where).

- A jolt of screen shake (`JOLT_SECS`) when any fault starts.
- A pulsing red full-screen tint while any fault is active, deeper and faster as the most urgent fault nears failure (`MIN_TINT` → `MAX_TINT`). Urgency is `Vitals::urgency`: the oxygen or heat pool a breach or loose bolts drain, the drift's own clock.
- Growing shake over the last half of the way to the most urgent failure (`CRITICAL_SHAKE`).
- The HUD's vitals panel (`ui`) says which system is failing; the alarm only says that one is.
- The screen shake and alarm flashing settings scale the shake and the pulse. With flashing at 0 the tint holds steady but still deepens with urgency.
- Frozen while paused (`not_paused`).

- **Resources:** `Alarm { level, active, jolt }` (`tint(t, flash)`, `shake()`).
- **Systems:** `spawn_tint` (Startup; UI at `GlobalZIndex(5)`), `reset` (`OnEnter(Playing)`), `update_alarm` (FixedUpdate/Resolve), `shake_camera` (writes `CameraRig::shake`), `tint_screen` (Update/Present). Shake jitter is a fixed function of time, not RNG.
- **Tests:** unit tests in `mod.rs`; integration tests in `tests/integration/alarm.rs`; e2e in `e2e/specs/outcome.spec.ts`.
- **Scenarios:** `scramble`, `breach`.
