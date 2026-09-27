# faults

What breaks, where, and how long until it is fatal. A fault is an entity with a `Fault { site, clock, elapsed, repair }`; spawning `fault_bundle(site, clock)` starts it, and each kind reacts to `Added<Fault>`. Clocks are independent and tick in FixedUpdate/Simulate; `resolve_faults` (FixedUpdate/Resolve) despawns repaired faults (`FaultFixed`) and reports expired ones (`FaultFailed`, which `level` turns into a loss). Every fix takes about 3 s.

| Kind | Sites (`sites.rs`) | Fix | Unfixed |
|---|---|---|---|
| `LooseBolts` (`bolts.rs`) | 4 engine panels × 3 bolts (the long faces of the engine blocks) | stand, click each loose bolt with the snapped wrench (0.9 s turn each) | engine overheats |
| `HullBreach` (`breach.rs`) | 6 airlock + 3 main hull wall points (airlock weight 3, hull 1) | stand and hold tape on the hole for `SEAL_SECS` (3 s) | oxygen runs out |
| `TrajectoryDrift` (`drift.rs`) | the nav helm | E at the pilot seat locks in; WASD nudges the drifting marker into the centre band and holds it `HOLD_SECS` (3 s; progress drains outside) | asteroid collision |

- **Sites (`sites.rs`):** positions come from the ship map: breach points are the marks `'1'..'9'` on hull walls, bolt panels the long faces of the engine blocks `'P'` / `'S'` (inner faces look toward the spine), the helm the cockpit's pilot seat. Names are ship-relative (port/starboard, fore/aft) so they hold whichever way the ship points; `label()` (the diagnostic list) is worked out from the layout ("left wall, bottom", "left engine, spine side", "hatch seal, left").
- **Types:** `FaultKind` (names, failure text, default clock range within `CLOCK_LIMITS` 55–90 s), `Site` (kind, room, position, normal, wall cell, weight, bolt positions), `Fault`, `FaultFixed`, `FaultFailed`, `Bolt`, `Nav` (marker, drift heading, helm engaged).
- **In-room cues (warm colours are for these only):** loose bolts stick out orange and rattle, the engine block glows hotter with urgency, a breach turns its wall tile into a torn hole (space shows through) venting a pulsing air ring, the nav marker turns red and the display frame blinks (held red when alarm flashing is off). None of this is visible from another room. A sealed breach becomes a strapped patch tile for the rest of the run. All of it freezes while paused.
- **Systems:** `tick_fault_clocks`, `resolve_faults`; bolts: `spawn_bolts`, `tighten_all`, `loosen_bolts`, `tighten_bolts`, `draw_bolts`, `heat_engines`; breach: `open_breaches` (breach tile), `seal_breaches`, `patch_sealed` (patch tile, after `resolve_faults`), `draw_breaches`; drift: `spawn_helm`, `use_helm` (FixedUpdate/Act), `start_drift` (drift heading from `GameRng`), `steer`, `draw_nav`.
- **Tests:** unit tests in each file; integration tests in `tests/integration/faults.rs` and `tests/integration/tools.rs`; e2e in `e2e/specs/faults.spec.ts`.
- **Scenarios:** `bolts`, `breach`, `drift`, `scramble`, `breach_critical`, `tape_low`.
