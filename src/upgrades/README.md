# upgrades

Between levels the engineer picks one of three upgrades (the `menu`'s upgrade screen, after CONTINUE on a landed level's end screen). They are kept for the rest of the campaign, and picking the same one again stacks it. This is where a roguelike choice of upgrades will grow; for now the same three are always offered.

| Upgrade | Per pick | Where it applies |
|---|---|---|
| `RunFaster` | +15% walking speed (`SPEED_PER_PICK`) | `player::move_player` |
| `FasterWrench` | each bolt turns in 75% of the time (`WRENCH_TIME_PER_PICK`) | `tools::use_wrench` (`WrenchTurn::secs`) |
| `WiderTape` | tape counts toward a breach from 50% further away and seals it 50% faster (`TAPE_PER_PICK`); one more strip laid side by side | `faults::breach::seal_breaches`, `tools::use_tape` (`strips_across`) |

- **One pick per level:** `Upgrades::choose(level, upgrade)` records the pick made on the way into `level` (2 to 5), dropping any pick for that level or later first, so flying a level again and continuing never stacks extra upgrades. At most `MAX_PICKS` (4). Fastest possible engineer: 1.6 x `PLAYER_SPEED`, still never waiting for a door (`ship::doors` test).
- **Not saved:** a new game (PLAY on the main menu, `MenuAction::NewGame`) starts without any. Losing and flying again keeps them.
- **Types:** `Upgrade` (`ALL`, `as_str`, `label`, `describe`), `Upgrades` (resource: `picks`, `count`, `choose`, `speed_factor`, `wrench_secs`, `tape_factor`, `all_of`).
- **Tests:** unit tests in `mod.rs`; integration tests in `tests/integration/upgrades.rs` (each upgrade through the real controls) and `tests/integration/campaign.rs` (picking them); e2e in `e2e/specs/campaign.spec.ts`.
- **Scenarios:** `level_five` and `final_landing` (four picks: run faster, faster wrench, wider tape, run faster).
