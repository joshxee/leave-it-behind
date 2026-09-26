# Derelict Ship — prototype grid tiles

64 modular **64 × 64** pixel tiles for the engineer's ship. The art shares the character's exact 16-entry palette: 15 cold blue/grey colours plus transparency. Broad metal forms, worn edges, patched plates, exposed services and cyan displays carry the same industrial theme. There are no decorative warm colours.

Open `preview/index.html` for the assembled room, door animation, diagnostic power, breach/patch comparison, grid overlay, zoom and tile library. The preview works offline. `preview/room.png` and `preview/room-motion.gif` are ready-to-view examples, with the engineer included for scale.

## Tile inventory

| Group | Tiles | Contents |
|---|---:|---|
| Floors | 9 | Three worn panels, grate, horizontal/vertical conduit, hatch, reinforced plate and drain |
| Floor edges | 8 | Four half tiles and four quarter tiles for the hull perimeter |
| Space | 2 | Quiet starfield backgrounds |
| Connecting walls | 16 | Every N/E/S/W connection mask: pillar, end caps, corners, straights, T junctions and cross |
| Damaged walls | 4 | Horizontal/vertical breaches, plus their strapped patch variants |
| Doors | 10 | Four opening frames and a locked state for each wall orientation |
| Cockpit | 6 | A 3 × 2 module: three-pane viewport, wraparound consoles, radar and pilot seat |
| Diagnostic screen | 3 | Two live waveform frames and powered-off state |
| Equipment | 6 | Pilot chair, console, locker, crate, oxygen rack and pipe stack |
| **Total** | **64** | |

## Files

- `png/`: 64 individual indexed PNGs. Floors and space are opaque; structure, floor edges and equipment use binary transparency.
- `ship-atlas.png`: untrimmed **512 × 512** atlas, 8 columns × 8 rows, no margins or spacing.
- `ship.aseprite`: editable indexed artwork in 64 cels, with door and screen animation tags and a fixed grid-cell pivot.
- `ship-atlas.aseprite.json`: native atlas export data.
- `ship-16.gpl` / `ship-16.pal`: exact copies of the engineer palette. GPL and JASC store RGB only; designate entry 0 transparent when importing. PNG and Aseprite already preserve alpha.
- `manifest.json`: tile IDs, categories, filenames, atlas rectangles, wall connections, collision boxes, interaction points and animation timing.
- `cockpit-module.png`: intact **192 × 128** cockpit master, in addition to its six 64px cells.
- `ship.tsj`: Tiled tileset with collision objects and useful tile properties.
- `example-room.tmj`: four-layer orthogonal Tiled sample room.
- `example-room.json`: the same room as simple layer arrays, plus dynamic cells and the preview character placement.
- `validation.json`: palette, image, atlas, collision, seam and module checks.
- `source/`: reproducible Aseprite Lua build and preview, metadata packaging, validation and optional local preview server.

## Grid and Bevy placement

All tile images retain the full 64px cell. Use a **(32, 32)** pixel anchor, equivalent to a centered Bevy sprite. Atlas column is `atlas_index % 8`; row is `atlas_index / 8` using integer division. Use nearest-neighbour sampling, disable tile flipping, and avoid per-tile trimming. Integer display scaling keeps pixels crisp.

For map coordinates with X right and Y down, place a tile center at `(x * 64 + 32, -(y * 64 + 32))`, then apply your world origin and scale. Draw layers in this order: **space → floor → structure → props → character**. This prototype uses flat compositing; foreground-wall occlusion can be added in the game if desired.

The character keeps its existing ground pivot **(32, 38)**. It therefore uses a different anchor from the tile cells. Do not recenter the engineer to match the tile pivot.

The provided collision arrays are tile-local rectangles `[x, y, width, height]`, measured from the top left. Convert a rectangle center into local Bevy coordinates with `(x + width/2 - 32, 32 - y - height/2)`. The rectangles describe prototype solid footprints, excluding decorative shadows. Floors and the exterior hazard state should be handled separately from those solid colliders.

## Connecting walls and floor edges

Wall connection bits are **N=1, E=2, S=4, W=8**. Compute the bitmask from neighbouring structure cells, then choose the corresponding `wall_*` ID. Doors and damaged segments count as structure neighbours. The core wall footprint is 24px wide, centered in the cell, with a dark underside beneath the top plate.

Examples: `wall_ne` joins north and east; `wall_ew` is horizontal; `wall_new` joins north, east and west. `wall_pillar` has no connections. All connected borders match, including straight-to-corner, junction, door and breach joins.

Floor-edge suffixes name the **outside** of the hull. `floor_edge_n` clears the northern half and keeps the southern half; `floor_edge_ne` keeps the southwest quarter. Their cut line is the wall centerline. Lay them beneath perimeter walls so a breach reveals space instead of a full square of floor sticking out beyond the hull.

## Doors, screens and breaches

`door_ew_*` sits in a wall running east–west; people pass north–south. `door_ns_*` sits in a wall running north–south; people pass east–west.

Play door frames **0 → 1 → 2 → 3**, 100 ms each, once to open. Hold frame 3. Reverse to close. The locked tile is a separate static variant with a crossed center marking. Only frame 3 has `passable: true`; transition frames retain a closed collision barrier. The clear aperture is 36px wide, suitable for an engineer ground collider no wider than 32px. Tiled does not automatically loop door transitions; their timing and playback intent are in the manifest and Aseprite tags.

The diagnostic screen loops `diagnostic_0` and `diagnostic_1` at 350 ms per frame. `diagnostic_2` is off. Its interaction point is at the console's near edge. Tiled includes the live screen animation.

Breaches preserve the intact wall's connection ports. Their serrated gap, peeled metal ribs and exposed cable indicate damage without introducing warm decorative colours. `vacuum_hazard: true` marks an unrepaired segment. Swapping to `wall_patched_*` closes the central collider and clears that flag. The patched tile shows a temporary bolted plate secured with grey repair straps; it is a visual repair-state prototype.

## Cockpit assembly

Place the six cells exactly as follows, facing north:

```text
cockpit_0_0  cockpit_1_0  cockpit_2_0
cockpit_0_1  cockpit_1_1  cockpit_2_1
```

The top row replaces three horizontal hull-wall cells. Its two outer ports join `wall_ew` without a seam. The lower row overlays interior floor. The six tiles are pixel-identical slices of the supplied master; do not flip individual cells.

## Rebuild

Run from the repository root with the existing sprite-axi and Aseprite installation:

```powershell
sprite-axi run assets/environment/derelict-ship/source/build.lua
node assets/environment/derelict-ship/source/package.cjs
sprite-axi run assets/environment/derelict-ship/source/preview.lua
node --preserve-symlinks --preserve-symlinks-main assets/environment/derelict-ship/source/validate.cjs
```

The build copies the palette from `assets/characters/engineer`; packaging copies one engineer PNG for the room preview. The validator reuses sprite-axi's installed `pngjs` decoder; set `PNGJS_PATH` to another installed copy if needed. The optional preview server binds only localhost: `node assets/environment/derelict-ship/source/serve.cjs`, then open `http://127.0.0.1:4174/preview/index.html`.

The artwork has been checked at its native pixel scale in the assembled room. Automated validation checks all 64 tiles and 240 compatible wall-border pairs. These are prototype assets and data; the Bevy game code has not been changed.
