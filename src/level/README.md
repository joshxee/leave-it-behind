# level

The flight: countdown to arrival, the level's fault schedule, and the outcome. `AppState::Landed` when the countdown hits zero (victory), `AppState::Lost` when any fault's clock runs out. R restarts from either screen (a fresh run: `OnEnter(Playing)` despawns every `RunEntity` and resets every run resource).

- **Level format (`def.rs`):** `LevelDef { duration_secs, seed, envelope, slots }`. A `FaultSlot` has a timing `window`, weighted `choices` (kind, optional pinned site) and a failure-`clock` window. `roll(&mut GameRng)` turns slots into `PlannedFault { at, site, clock }`; pinned values draw no randomness. `validate()` checks durations, weights, site kinds, windows and clocks (45–60 s), and that the worst case over every possible roll stays inside the `Envelope { max_overlap, response_secs }` (faults needing handling at once, each busy for `response_secs`). A slot whose site is still busy waits until it is free.
- **Level one (`one.rs`):** fully pinned (every slot exact, `seed` fixes drift headings): 4:00 flight, 10 faults. 0:10–1:30 one at a time (each kind once), 1:36–2:40 two at once, 2:56–3:06 all three at once, spread tip to tail. Needs 12 s of the 20 s tape roll.
- **Resources:** `CurrentLevel`, `Journey { duration, elapsed }`, `FaultPlan { pending }`, `RunStats { started, fixed, failure }`.
- **Systems:** `start_run` (`OnEnter(Playing)`: reseed, roll, reset), `advance_journey` → `start_due_faults` (FixedUpdate/Simulate), `count_fixes` → `decide_outcome` (FixedUpdate/Resolve, after `resolve_faults`), `restart` (Update, in `Landed`/`Lost`).
- **Functions:** `format_clock`, `peak_overlap`.
- **Tests:** unit tests in `def.rs`, `one.rs`, `mod.rs`; integration tests in `tests/integration/level.rs` (schedule, determinism, landing, loss, a full run fixing every fault in 20 s, restart); e2e in `e2e/specs/outcome.spec.ts`.
- **Scenarios:** `default`, `landing`, `breach_critical`.
