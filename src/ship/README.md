# ship

The ship as data, drawn with the derelict-ship tiles, and the camera that frames one room at a time. The ship points up: **cockpit** (nav helm) at the top, then the **engineer's quarters** (diagnostic console, player spawn), **engine room** (two engine blocks), **main hull** (cargo), and the **airlock** (locked outer hatch) at the bottom. A conduit, the spine, runs through every door. Helm to airlock is about 9.5 s at `PLAYER_SPEED`; neighbouring rooms' centres are about 2 s apart.

## The map

`map.rs` holds `SHIP`, an ASCII grid of 64-pixel cells (legend in its module docs). `ShipMap::parse` derives everything else and rejects a layout that breaks the rules: each room is walled off (flood fill from its marker `C` `Q` `E` `H` `A`), a door joins two rooms in a straight wall, the hatch and every breach point sit in a straight hull wall. **Nothing else hard-codes a position or the ship's orientation**: to change the layout, or turn the ship, edit `SHIP`. The unit tests check the result (rooms fit the view, every room reachable, the spine is clear, every fault site workable from open floor), and a small sideways ship in the tests proves nothing assumes an axis.

- Derived: rooms and their `interior()` / `frame()` (what the camera shows: the floor and the solid core of the room's own walls, plus the cockpit window row), `room_at(p)` (rooms meet at a wall's centre line), wall masks for the auto-joining wall tiles, merged `walls()` and `props()` colliders from the tiles' collision boxes, `floor_quadrants` (the hull outline, exact even at inner corners: a flood fill of "outside" over a 3x3 block grid per cell), `floor_tile` (per-room floor patterns), `doors()`, `wall_mark` (breach points), `helm()`, `console_point()`, `block(ch)` (engines, bunk, nav display).
- `layout.rs`: the accessors the rest of the game uses (`walls`, `colliders`, `player_spawn`, `helm_seat`, `joystick`, `nav_screen`, `port_engine`, ...), the circle-vs-rectangle collision (`resolve_circle`, `move_circle`), and `wall_contact` (where a tool reaching out from the engineer meets a wall: always the face on their side, never the far one).

## Doors

`doors.rs`: a door slides open (four frames, 0.1 s each) when the engineer is within `DOOR_REACH` (3 cells, so a running engineer never waits) and closes behind them. Only frame 3 is passable: until then the leaf (`Door::leaf`, the 36-unit gap between the jambs) is solid, which is why the player's footprint is 28 units wide. `door_assist` steers an engineer heading into a door toward its centre line. The hatch is a locked door: part of the wall.

## Drawing

The maintenance pack supplies full-height wall faces and narrow caps pushed to the outer edge of each room. `depth.rs` selects one of 16 joining masks × 16 room-facing variants, sorts the engineer and engines by ground contact, and places door art on the same border. All foreground walls retain the same 22-unit height. The map collision cores remain a walking guard band. Floor art extends beneath the narrower caps; curtains include the raised silhouette. Breaches and tape use `wall_art_point` so holes, strips and the tape feed meet on the same surface.

- **Resources:** `CurrentRoom`, `Colliders` (walls + props; door leaves are added by whoever moves the player), `Walls` (tape sticks to these).
- **Components:** `CameraRig { anchor, shake }`, `EngineBlock` (tinted by the bolts fault), `WallCell`, `Door`.
- **Systems:** `spawn_camera`, `spawn_ship`, `spawn_doors` (Startup); `reset_room`, `close_doors` (`OnEnter(Playing)`); `operate_doors` (FixedUpdate/Move), `track_room` (FixedUpdate/Act); `frame_camera`, `place_curtains`, `draw_doors`, depth presentation systems (Update/Present).
- **Functions:** `cursor_to_world` (matches the camera's `ScalingMode::AutoMin` 1408×792 world view within a 1280×720 window), `room_frame`, `step_door`, `door_assist`, `leaf_rect`.
- **Tests:** unit tests in `map.rs`, `layout.rs`, `doors.rs`, `mod.rs`; integration tests in `tests/integration/player.rs` (room cut, traversal time) and `tests/integration/doors.rs` (opening ahead of the engineer, closed doors block, steering into the doorway, the locked hatch); e2e in `e2e/specs/controls.spec.ts`.
- **Scenarios:** `quiet` (walk anywhere without faults).
