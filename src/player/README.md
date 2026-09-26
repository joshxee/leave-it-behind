# player

The engineer: a circle that walks with WASD (sliding along walls), faces the mouse cursor, and uses whichever console is in reach with E.

- **Components:** `Player`, `Facing` (unit vector toward the cursor), `Locked` (strapped in at the helm: no walking), `Interactable { kind, range }` (on consoles; `InteractKind::{Helm, Diagnostics}`).
- **Resources/messages:** `PlayerIntent` (raw input read in `Update`, consumed by `FixedUpdate`: WASD direction, cursor world point, left button held/clicks, E presses, number-key slot, wheel steps), `Focus` (console in reach), `InteractPressed { target }` (once per tick with an E press).
- **Systems:** `spawn_player` (`OnEnter(Playing)`, `RunEntity`), `read_input` (Update/Input), `move_player` → `face_aim` → `update_focus` → `send_interact` (FixedUpdate/Move), `point_visor` (Update/Present).
- **Pure functions:** `direction_from_keys`, `slot_from_keys`, `scroll_steps`, `facing_toward`, `nearest_interactable`.
- **Tests:** unit tests in `mod.rs`; integration tests in `tests/integration/player.rs`; e2e in `e2e/specs/controls.spec.ts`.
- **Scenarios:** `default`, `quiet`.
