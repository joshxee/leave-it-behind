# tools

Hand tools held visibly at the end of the engineer's arm, pointing where they face. `1`/`2` (or the scroll wheel) pick one; left click uses it. No menus.

- **Wrench:** the head rests `WRENCH_REACH` ahead and snaps (soft magnet) to any `WrenchTarget` within `SNAP_RADIUS`; it turns green when snapped. A click on a snapped target turns it for `WRENCH_TURN_SECS` (0.9 s), then sends `WrenchTightened`. Moving the head off the target cancels the turn.
- **Tape:** while the button is held and the roll is within `TAPE_CONTACT` of any wall, tape comes off the roll (1 s per s, `TAPE_CAPACITY` = 20 s per run) as `TapeLaid { point, normal, secs }` plus visible `TapeStrip`s. Tape on bare wall is wasted; the roll shrinks as it empties.
- **Resources/messages:** `ToolBelt { held, tape_left }`, `ToolState { tip, head, snap, turn, taping }`, `WrenchTightened`, `TapeLaid`.
- **Components:** `WrenchTarget { pos }` (put on loose bolts by `faults::bolts`), `TapeStrip`, private `ToolPart` visuals.
- **Systems:** `reset_tools` (`OnEnter(Playing)`), `select_tool` → `aim_tool` → `use_wrench` → `use_tape` (FixedUpdate/Act), `draw_tools` (Update/Present).
- **Tests:** unit tests in `mod.rs`; integration tests in `tests/integration/tools.rs`; e2e in `e2e/specs/controls.spec.ts` and `e2e/specs/faults.spec.ts`.
- **Scenarios:** `bolts`, `breach`, `tape_low`.
