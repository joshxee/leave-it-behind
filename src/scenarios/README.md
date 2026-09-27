# scenarios

Fixtures that put the world into an exact situation. Shared by native tests (`test_app_with`), the e2e web build (`?scenario=<name>`), and `e2e/tools/capture.mjs --scenario`. A scenario applies on top of every fresh run (`OnEnter(Playing)`, `RunSet::Scenario`), so it also applies again after R. Any scenario skips the title screen; the plain boot (no scenario) is the title and menu situation.

- **Types:** `Scenario` (`ALL`, `name`, `from_name`, `apply`), `ActiveScenario`.
- **Scenarios:** `default` (level one), `quiet` (no faults ever), `bolts` (port engine, spine side), `breach` (airlock, port wall aft), `drift` (engineer on the spot, right tool in hand, facing the work), `diagnostics` (at the console with bolts and a breach active), `scramble` (all three kinds at once, engineer in the quarters), `landing` (3 s to arrival), `breach_critical` (breach with 2 s left), `tape_low` (breach with 1 s of tape), `paused` (level one with the pause menu open), `settings` (paused, settings screen over the pause menu).
- **Where the engineer stands:** worked out from the map and the sprite's tool reach (`stand_off`), never hard-coded: the wrench's bite a few units short of the bolt (the snap pulls it on), the tape's edge pressed to the wall.
- **Helpers:** `bolts_stand`, `breach_stand`, `console_stand`.
- **Tests:** `tests/integration/scenarios.rs` (every scenario boots; setups are right).
- **Adding one:** see the template in `TESTING.md`.
