# tools

Hand tools, always in the engineer's hand (the sprite draws them). `1`/`2` (or the scroll wheel) pick one; left click uses it, **but only standing still**: walking cancels a wrench turn at once, stops tape coming off the roll, and clicks made while walking are dropped. No menus.

- **Where the tool touches:** `tool_tip` uses the sprite pack's per-direction contact points (`art::engineer::contact`), so the working end is exactly where the art draws the wrench's bite or the tape's edge: 16 facings standing, 8 walking.
- **Wrench:** the bite snaps (soft magnet) to any `WrenchTarget` within `SNAP_RADIUS`; a cyan ring marks the snapped bolt and shrinks as the turn goes round. A click on a snapped target turns it for `WRENCH_TURN_SECS` (0.9 s), then sends `WrenchTightened`. Moving off the target, or walking, cancels the turn.
- **Tape:** while the button is held, the engineer stands still and the roll's edge is within `TAPE_CONTACT` of any wall, tape comes off the roll (1 s per s, `TAPE_CAPACITY` = 20 s per run) as `TapeLaid { point, normal, secs }` plus six-frame woven `TapeStrip`s and a feed from the roll to the raised wall surface. Tape on bare wall is wasted; the HUD shows what is left.
- **Resources/messages:** `ToolBelt { held, tape_left }`, `ToolState { tip, head, snap, turn, taping }`, `WrenchTightened`, `TapeLaid`.
- **Components:** `WrenchTarget { pos }` (put on loose bolts by `faults::bolts`), `TapeStrip`, private `SnapRing`, `TapeApplication`, `TapeFeed`.
- **Systems:** `reset_tools` (`OnEnter(Playing)`), `select_tool` → `aim_tool` → `use_wrench` → `use_tape` (FixedUpdate/Act), `draw_snap`, `animate_tape`, `draw_tape_feed` (Update/Present).
- **Tests:** unit tests in `mod.rs`; integration tests in `tests/integration/tools.rs` (including walking cancels a turn, clicks while walking do nothing, tape stops while walking); e2e in `e2e/specs/controls.spec.ts` and `e2e/specs/faults.spec.ts`.
- **Scenarios:** `bolts`, `breach`, `tape_low`.
