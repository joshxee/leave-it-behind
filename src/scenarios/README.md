# scenarios

Fixtures that put the world into an exact situation. Shared by native tests (`test_app_with`), the e2e web build (`?scenario=<name>`), and `e2e/tools/capture.mjs --scenario`. A scenario applies on top of every fresh run (`OnEnter(Playing)`, `RunSet::Scenario`), so it also applies again after R.

- **Types:** `Scenario` (`ALL`, `name`, `from_name`, `apply`), `ActiveScenario`.
- **Scenarios:** `default` (level one), `quiet` (no faults ever), `bolts`, `breach`, `drift` (engineer on the spot, right tool in hand), `diagnostics` (at the console with bolts and a breach active), `scramble` (all three kinds at once, engineer in the quarters), `landing` (3 s to arrival), `breach_critical` (breach with 2 s left), `tape_low` (breach with 1 s of tape).
- **Helpers:** `bolts_stand`, `breach_stand`, `console_stand`.
- **Tests:** `tests/integration/scenarios.rs` (every scenario boots; setups are right).
- **Adding one:** see the template in `TESTING.md`.
