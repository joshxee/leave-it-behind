# ui

HUD in the bundled Super Indie font (default font without an `AssetServer`, as in headless tests). ASCII only: the font has no other glyphs.

- Top centre: `ARRIVAL IN m:ss` (`LANDED` after touchdown). Top left: the current room.
- Bottom: context prompt (`E  take the helm`, `Click  turn the bolt`, ...; on a dark backing so it reads over the room's bottom wall, hidden when empty), the tool belt (`[1] WRENCH  [2] TAPE 20s`, held tool highlighted in visor cyan), and a controls hint for the first `HINT_SECS`.
- Colours from `palette.rs`: the pack's cold palette; red only for danger (the alarm tint, the lost screen's tint).
- The landed / lost screen (`EndScreen`, `GlobalZIndex(30)`): title, stats, "Press R to fly again".
- Never shows where a fault is (that is `diagnostics`).
- **Components:** `TimerText`, `RoomText`, `PromptText`, `BeltSlot`, `EndScreen`.
- **Systems:** `spawn_hud`, `spawn_end_screen` (Startup), `update_hud`, `update_prompt`, `update_end_screen` (Update/Present).
- **Functions:** `timer_label`, `belt_label`, `prompt`, `game_font`.
- **Tests:** unit tests in `mod.rs`; covered visually by every e2e spec's screenshots.
- **Scenarios:** `default`, `landing`, `breach_critical`.
