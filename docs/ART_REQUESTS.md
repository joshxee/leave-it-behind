# Art requests

What the game still draws as placeholder shapes, or needs from the next pass of the prototype art. Same rules as the packs: sprite-axi + Aseprite, the shared 16-entry palette (`assets/characters/engineer/engineer-16.gpl`), 64×64 cells, one fixed overhead light, 1 art pixel = 1 world unit. **Warm colours only for danger** (a fault and what it does): everything decorative stays in the cold palette.

## 1. Raised walls (the look we're leaning towards)

The depth study (`assets/environment/derelict-ship-depth-study`) shows the raised treatment, but it is one composed image. The game needs it as modular tiles, keeping today's collision footprints:

- Every connecting wall mask (16 variants), split into a **base** (the footprint, drawn under the engineer) and a **raised cap and face** (drawn over the engineer when they are behind the wall).
- A low **cutaway** version (the study's 6 px) for walls between the camera and the room, so the near wall never hides the engineer.
- The **doors** (four opening frames and locked), the **breach** and **patched** walls, and the **cockpit module** in the same treatment.

Game side, once the tiles exist: depth-sort the engineer against the raised pieces by their ground line. Rendering is already layered (space, floor, structure, props, characters), so this is a contained change.

## 2. Placeholders in the game today

| Thing | Where | Size in the current map | Notes |
|---|---|---|---|
| Engine block | engine room, `P` / `S` | 2 × 6 cells each | The long sides are the bolt panels (three bolts each). Needs a way to show heat (an overlay or hot variant): the loose-bolts cue. |
| Bolt heads | on the engine panels | about 14 px | Tight (flush) and loose (sticking out). Loose ones are danger and may be warm. |
| Bunk | quarters, `b` | 2 × 1 cells | |
| Nav display | cockpit, `N` | 3 × 3 cells, on the floor | Shows a drifting marker, the centre band and a hold bar (drawn in code on top, so the art is the housing and screen). |
| Joystick | on the pilot console | about 24 px | Tilts with the player's input while at the helm. |
| Tape strips | on walls | 24 × 8 px | The tape laid while sealing (the patched wall tile already exists). |
| Snap ring | round a bolt | 26 px | The wrench's target marker; could stay a UI shape. |

## 3. Engineer

- **Tape tool readability:** at game size the tape pose reads like a ring spanner, next to the similar grey wrench. A chunkier roll, or a palette accent on it, would tell them apart. (The drawn roll can't show how much tape is left; the HUD does.)
- **At the helm:** a seated pose for the pilot chair. Today the engineer stands in the chair (the empty-handed idle pose, facing the window).
- **Housekeeping:** run the pack's build once with Aseprite to refresh `engineer.aseprite`, `preview/overview.png` and `preview/motion.gif`. The walk-with-tool frames (`wrench_walk`, `tape_walk`) were added with a stand-in for the Aseprite API; see `assets/characters/engineer/README.md`.
