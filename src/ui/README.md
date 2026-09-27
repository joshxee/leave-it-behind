# ui

HUD in the bundled Super Indie font (default font without an `AssetServer`, as in headless tests). ASCII only: the font has no other glyphs.

- Top centre: `ARRIVAL IN m:ss` (`LANDED` after touchdown). Top left: the current room.
- Bottom: context prompt (`E  take the helm`, `Click  turn the bolt`, ...; on a dark backing so it reads over the room's bottom wall, hidden when empty), the tool belt (`[1] WRENCH  [2] TAPE 20s`, held tool highlighted in visor cyan), and a controls hint for the first `HINT_SECS` (off in settings).
- Colours from `palette.rs`: the pack's cold palette; red only for danger (the alarm tint, the lost screen's tint).
- Shown during a flight and behind the end screen; hidden on the title and menus (`Hud`, `show_hud`). The end screen itself is a `menu` screen.
- The notice line (`Notices::push`): one short message at a time for `NOTICE_SECS`, above everything (`GlobalZIndex(60)`), timed in real time so it clears while paused. Used by `save` and `menu`.
- Never shows where a fault is (that is `diagnostics`).
- The coaching panel under the countdown (level one's first flight) belongs to `coach`.
- **Components:** `Hud`, `TimerText`, `RoomText`, `PromptText`, `BeltSlot`, `NoticeText`.
- **Resources:** `Notices` (`push`, `current`, `tick`).
- **Systems:** `spawn_hud`, `spawn_notice_line` (Startup), `show_hud`, `update_hud`, `update_prompt`, `show_notices` (Update/Present).
- **Functions:** `timer_label`, `belt_label`, `prompt`, `game_font`.
- **Tests:** unit tests in `mod.rs`; `tests/integration/menu.rs` (HUD hidden on menus); covered visually by every e2e spec's screenshots.
- **Scenarios:** `default`, `landing`, `breach_critical`.
