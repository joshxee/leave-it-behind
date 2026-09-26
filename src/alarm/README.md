# alarm

Ambient cues: the only warning outside the diagnostic screen. They say something is wrong and how urgent it is, never where.

- A jolt of screen shake (`JOLT_SECS`) when any fault starts.
- A pulsing red full-screen tint while any fault is active, deeper and faster as the most urgent fault's clock runs down (`MIN_TINT` → `MAX_TINT`).
- Growing shake over the last half of the most urgent fault's clock (`CRITICAL_SHAKE`).

- **Resources:** `Alarm { level, active, jolt }` (`tint(t)`, `shake()`).
- **Systems:** `spawn_tint` (Startup; UI at `GlobalZIndex(5)`), `reset` (`OnEnter(Playing)`), `update_alarm` (FixedUpdate/Resolve), `shake_camera` (writes `CameraRig::shake`), `tint_screen` (Update/Present). Shake jitter is a fixed function of time, not RNG.
- **Tests:** unit tests in `mod.rs`; integration tests in `tests/integration/alarm.rs`; e2e in `e2e/specs/outcome.spec.ts`.
- **Scenarios:** `scramble`, `breach`.
