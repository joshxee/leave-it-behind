# ui

HUD in the bundled Super Indie font (default font without an `AssetServer`, as in headless tests). ASCII only: the font has no other glyphs.

- Top centre: `ARRIVAL IN m:ss` (`LANDED` after touchdown). Top left: the current room. Under it: `LEVEL n OF 5` (`level_label`).
- Top right, the vitals panel (`vitals.rs`): `O2` and `HEAT` bars with a percentage (O2 reads 0% only when gone, heat 100% only when overheated) and the course, `COURSE OK` or `IMPACT IN m:ss` while the ship drifts. Always shown in cold colours; a row turns warm (`ALERT`) and pulses with the alarm flashing setting while a fault of its kind is active. It says what is failing and how soon, never where.
- Bottom: context prompt (`E  take the helm`, `Click  turn the bolt`, ...; on a dark backing so it reads over the room's bottom wall, hidden when empty), the tool belt (`[1] WRENCH  [2] TAPE 20s`, held tool highlighted in visor cyan), and a controls hint for the first `HINT_SECS` (off in settings).
- Colours from `palette.rs`: the pack's cold palette; red only for danger (the alarm tint, the lost screen's tint).
- Shown during a flight and behind the end screen; hidden on the title and menus (`Hud`, `show_hud`). The end screen itself is a `menu` screen.
- The notice line (`Notices::push`): one short message at a time for `NOTICE_SECS`, above everything (`GlobalZIndex(60)`), timed in real time so it clears while paused. Used by `save` and `menu`.
- Never shows where a fault is (that is `diagnostics`).
- The coaching panel under the countdown (level one's first flight) belongs to `coach`.
- **Components:** `Hud`, `TimerText`, `RoomText`, `LevelText`, `PromptText`, `BeltSlot`, `NoticeText`, `GaugeLabel`, `GaugeValue`, `GaugeFill` (each holding its `Gauge`: `Oxygen`, `Heat`, `Course`).
- **Resources:** `Notices` (`push`, `current`, `tick`).
- **Systems:** `spawn_hud`, `spawn_vitals`, `spawn_notice_line` (Startup), `show_hud`, `update_hud`, `update_vitals`, `update_prompt`, `show_notices` (Update/Present).
- **Functions:** `timer_label`, `level_label`, `belt_label`, `prompt`, `game_font`, `gauge` (what a vitals row says).
- **Tests:** unit tests in `mod.rs` and `vitals.rs`; `tests/integration/menu.rs` (HUD hidden on menus), `tests/integration/vitals.rs` (the panel's rows); e2e in `e2e/specs/vitals.spec.ts`, and covered visually by every e2e spec's screenshots.
- **Scenarios:** `default`, `landing`, `breach_critical`, `second_breach`, `drift`.
