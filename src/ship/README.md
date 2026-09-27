# ship

The ship's geometry and the room camera. Five rooms along a corridor spine (y = 0), front (+x) to tail (−x): **cockpit** (nav helm), **engineer's quarters** (diagnostic console, player spawn), **engine room** (two engine blocks), **main hull** (cargo), **airlock** (hatch). Bulkheads have a door gap on the spine. Helm to airlock is about 15 s at `PLAYER_SPEED`; each room hop 2.5–4 s.

- **`layout.rs` (pure data, unit tested):** `RoomId` (`interior`, `center`, `at(x)`), `walls()`, `props()`, `colliders()`, prop positions (`helm_seat`, `joystick`, `nav_screen`, `diag_console`, engines, crates, bunk), `resolve_circle` / `move_circle` (circle-vs-rectangle sliding collision), `wall_contact` (for tape).
- **Resources:** `CurrentRoom`, `Colliders` (walls + props), `Walls` (walls only).
- **Components:** `CameraRig { anchor, shake }` (the camera; aiming uses `anchor`, the alarm writes `shake`), `EngineBlock` (tinted by the bolts fault), private `Curtain`s.
- **Systems:** `spawn_camera`, `spawn_ship` (Startup), `track_room` (FixedUpdate/Act), `frame_camera` (hard cut to the current room), `place_curtains` (void-colored sprites hide every other room) (Update/Present), `reset_room` (`OnEnter(Playing)`).
- **Functions:** `cursor_to_world` (matches the camera's `ScalingMode::AutoMin` 1280×720 projection).
- **Tests:** unit tests in `layout.rs` and `mod.rs`; integration tests in `tests/integration/player.rs` (room cut, traversal time); e2e in `e2e/specs/controls.spec.ts`.
- **Scenarios:** `quiet` (walk anywhere without faults).
