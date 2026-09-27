# player

The engineer: walks with WASD (sliding along walls, steered into doorways), faces the mouse cursor, and uses whichever console is in reach with E. Drawn with the engineer sprite pack (`assets/characters/engineer`).

- **Components:** `Player`, `Facing` (unit vector toward the cursor), `Movement { walking, heading }` (walking = WASD held and free to move; tools only work while not walking), `Locked` (strapped in at the helm: no walking), `Interactable { kind, range }` (on consoles; `InteractKind::{Helm, Diagnostics}`), `EngineerSprite` (on the sprite child: the current animation, its facing and clock).
- **Resources/messages:** `PlayerIntent` (raw input read in `Update`, consumed by `FixedUpdate`: WASD direction, cursor world point, left button held/clicks, E presses, number-key slot, wheel steps), `Focus` (console in reach), `InteractPressed { target }` (once per tick with an E press).
- **Footprint:** a circle of `PLAYER_RADIUS` (14) at the engineer's feet, the sprite's ground point (`art::engineer::ROOT`), so a door's 36-unit gap fits.
- **Sprite (`sprite.rs`):** one animation at a time, cut cleanly (a new one starts at its first frame): `wrench_walk` / `tape_walk` while walking (tool in hand, facing the way they walk, 8 directions), `wrench_hold` / `tape_hold` standing (facing the mouse, 16 directions, or the bolt the wrench is snapped to), `wrench_use` over a bolt's turn and `tape_use` while tape goes on (facing latched when the use starts), `idle` at the helm (facing the window). The sprite leans up to `LEAN_MAX` so the wrench's bite lands on a snapped bolt.
- **Systems:** `spawn_player` (`OnEnter(Playing)`, `RunEntity`, with the sprite child), `read_input` (Update/Input), `move_player` (adds closed door leaves and `door_assist`) → `face_aim` → `update_focus` → `send_interact` (FixedUpdate/Move), `sprite::animate` (Update/Present).
- **Pure functions:** `direction_from_keys`, `slot_from_keys`, `scroll_steps`, `facing_toward`, `nearest_interactable`, `sprite::pose`, `sprite::frame`.
- **Tests:** unit tests in `mod.rs` and `sprite.rs`; integration tests in `tests/integration/player.rs` and `tests/integration/doors.rs`; e2e in `e2e/specs/controls.spec.ts`.
- **Scenarios:** `default`, `quiet`.
