# player

Player square moved with the arrow keys (300 px/s, clamped to the 1280×720 arena).

- **Components/resources:** `Player`, `PlayerIntent` (input read in `Update`, consumed in `FixedUpdate`).
- **Systems:** `spawn_camera`, `spawn_player` (Startup, color from `GameRng`), `read_input` (Update/Input), `move_player` (FixedUpdate/Simulate).
- **Pure functions:** `direction_from_keys`, `step_position`.
- **Tests:** unit tests in `mod.rs`; integration tests in `tests/integration/player.rs`; e2e in `e2e/specs/interaction.spec.ts`.
- **Scenarios:** `default`.
