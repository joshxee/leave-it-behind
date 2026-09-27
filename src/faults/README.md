# faults

What breaks, where, and how long until it is fatal. A fault is an entity with a `Fault { site, clock, elapsed, repair }`; spawning `fault_bundle(site, clock)` starts it, and each kind reacts to `Added<Fault>`. Clocks are independent and tick in FixedUpdate/Simulate; `resolve_faults` (FixedUpdate/Resolve) despawns repaired faults (`FaultFixed`) and reports expired ones (`FaultFailed`, which `level` turns into a loss). Every fix takes about 3 s.

| Kind | Sites (`sites.rs`) | Fix | Unfixed |
|---|---|---|---|
| `LooseBolts` (`bolts.rs`) | 4 engine panels × 3 bolts | click each loose bolt with the snapped wrench (0.9 s turn each) | engine overheats |
| `HullBreach` (`breach.rs`) | 6 airlock + 3 main hull wall points (airlock weight 3, hull 1) | hold tape on the hole for `SEAL_SECS` (3 s) | oxygen runs out |
| `TrajectoryDrift` (`drift.rs`) | the nav helm | E at the joystick locks in; WASD nudges the drifting marker into the centre band and holds it `HOLD_SECS` (3 s; progress drains outside) | asteroid collision |

- **Types:** `FaultKind` (names, failure text, default clock range within `CLOCK_LIMITS` 55–75 s), `Site` (kind, room, position, normal, weight, bolt positions), `Fault`, `FaultFixed`, `FaultFailed`, `Bolt`, `Nav` (marker, drift heading, helm engaged).
- **In-room cues:** loose bolts stick out and rattle, the engine block glows hotter with urgency, a breach vents a pulsing air ring, the nav marker turns red and the display frame blinks. None of this is visible from another room.
- **Systems:** `tick_fault_clocks`, `resolve_faults`; bolts: `spawn_bolts`, `tighten_all`, `loosen_bolts`, `tighten_bolts`, `draw_bolts`, `heat_engines`; breach: `open_breaches`, `seal_breaches`, `draw_breaches`; drift: `spawn_helm`, `use_helm` (FixedUpdate/Act), `start_drift` (drift heading from `GameRng`), `steer`, `draw_nav`.
- **Tests:** unit tests in each file; integration tests in `tests/integration/faults.rs` and `tests/integration/tools.rs`; e2e in `e2e/specs/faults.spec.ts`.
- **Scenarios:** `bolts`, `breach`, `drift`, `scramble`, `breach_critical`, `tape_low`.
