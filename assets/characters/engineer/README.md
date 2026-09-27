# Derelict Engineer

A bulky lone engineer in a worn blue-grey pressure suit: broad helmet, cyan inset visor, ribbed joints, repaired shoulder, scuffed crown plates and a vented backpack. The artwork uses a high-angle top-down view with one fixed light directly overhead. There are no warm accents.

Open **preview/index.html** for mouse aiming, animation playback, individual frame selection, dark/light floor comparison, contact guides, and a gallery of every PNG. It works offline. The game-size strip uses 64 CSS pixels per sprite and scrolls on narrow screens. Enlarged views use nearest-neighbour scaling.

## Contents

| Action | Directions | Frames per direction | PNGs | Timing |
|---|---:|---:|---:|---|
| Idle | 16 at 22.5° | 1 | 16 | Hold |
| Walk | 8 at 45° | 4 | 32 | 110 ms each; 440 ms loop |
| Wrench held | 16 at 22.5° | 1 | 16 | Hold |
| Wrench use | 8 at 45° | 3 | 24 | 90 ms each; 270 ms once |
| Tape held | 16 at 22.5° | 1 | 16 | Hold |
| Tape use | 8 at 45° | 3 | 24 | 90 ms each; 270 ms once |
| Walk with wrench | 8 at 45° | 4 | 32 | 110 ms each; 440 ms loop |
| Walk with tape | 8 at 45° | 4 | 32 | 110 ms each; 440 ms loop |
| **Total** | | | **192** | |

- `png/`: individual transparent **64 × 64** PNGs, with no trimming or antialiasing.
- `engineer-16.gpl` and `engineer-16.pal`: GIMP/Aseprite and JASC palettes. **16 entries total: index 0 transparent, 15 opaque cold colours.** All 15 opaque colours are used across the set. GPL/JASC store RGB only; index 0 must be designated transparent when importing. PNG and Aseprite already store the transparency.
- `engineer.aseprite`: editable indexed artwork, flattened to one editable pixel layer per cel, with correctly bounded animation tags and a root slice. It still holds the original 128 frames and 72 tags until the pending rebuild (see **Rebuild and audit**); the build then writes all 192 frames and 88 tags.
- `manifest.json`: filenames, angles, timing, atlas indices, root, tool contacts and animation groups.
- `engineer-atlas.png`: 1024 × 768 atlas, 16 columns × 12 rows, 64 × 64 cells. The game loads this file. All cells are untrimmed.
- `engineer-atlas.aseprite.json`: native Aseprite atlas metadata.
- `preview/overview.png`: enlarged examples plus native-size dark/light floor checks.
- `preview/motion.gif`: simultaneous eight-direction walk and tool-use proof on both floors.
- `validation.json`: automated delivery audit.
- `source/`: reproducible native Aseprite Lua artwork and preview construction, plus PNG validation and preview packaging helpers.

Naming: `engineer_<action>_<direction>_<frame>.png`. Frames are zero-based. Directions are ordered clockwise in image coordinates:

`E, ESE, SE, SSE, S, SSW, SW, WSW, W, WNW, NW, NNW, N, NNE, NE, ENE`.

## Placement in Bevy

The root is **(32, 38)** pixels measured from the image's top-left corner. Every exported image keeps its full canvas. This root is the standing ground point; the helmet is above it in the top-down projection. The torso stays fixed while limbs carry the walk cycle.

Set the sprite's normalized anchor to **(0.0, -0.09375)**, where Bevy's anchor coordinates are positive upward. Equivalently, if using a centered sprite child, place its center **6 pixels above** the entity's ground point. Multiply pixel offsets by your uniform sprite scale. Do not trim or recenter frames independently.

Use nearest texture sampling and integer display scaling. Keep the sprite quad unrotated and select the facing frame from the mouse aim angle. All shading is authored for the fixed overhead light. If you additionally rotate a chosen quad, the facing would be applied twice.

For a Bevy world-space aim vector `(dx, dy)` with positive Y upward, this pure Rust selector produces the direction index used by the manifest:

```rust
fn direction_index(dx: f32, dy: f32, directions: usize) -> usize {
    let clockwise = (-dy).atan2(dx);
    let step = std::f32::consts::TAU / directions as f32;
    (clockwise / step).round().rem_euclid(directions as f32) as usize
}
```

Retain the previous facing when the aim vector is zero. Use 16 directions for idle/held tools, and 8 for walking/tool use. The eight-direction index multiplied by two is the manifest's `direction_index`. Select the walk direction from movement; standing and tool poses follow aim. A separate set for strafing while aiming in another direction is outside this asset set.

| Action | Zero-based atlas index |
|---|---|
| Idle | `dir16` |
| Walk | `16 + dir8 * 4 + frame` |
| Wrench held | `48 + dir16` |
| Wrench use | `64 + dir8 * 3 + frame` |
| Tape held | `88 + dir16` |
| Tape use | `104 + dir8 * 3 + frame` |
| Walk with wrench | `128 + dir8 * 4 + frame` |
| Walk with tape | `160 + dir8 * 4 + frame` |

## Tool contact and use events

Each tool frame supplies `contact_pixels: [x, y]`. For the wrench this is the center of the open bite; the jaws extend a few pixels beyond it. For tape it is the leading application edge. The tool is the outermost extension along the aim direction, near the canvas border. The whole head/roll is preserved, including diagonals.

The contact is identical between a held pose and every use frame for a given tool and direction. Reach varies with direction to fit the square canvas, so use the per-direction coordinate rather than a constant reach radius. In Bevy coordinates, the contact offset from the root is:

```text
contact_offset = (x - 32, 38 - y) * sprite_scale
contact_world  = engineer_root_world + contact_offset
```

Compare that point with the bolt/wall interaction point, or compute `engineer_root_world = interaction_point - contact_offset` when placing a snapped pose. Quantize to the use direction before computing the offset, then latch that direction throughout the short use animation. This prevents the tool contact moving as the mouse crosses a direction boundary.

Use frame **1** (the middle frame) for the repair/application event. Wrench frames rock the handle about the bite and return; tape frames pull the tab, press with the spare glove, and recover. Play `0 → 1 → 2` once, then return to the held pose. Walking and tool use are mutually exclusive.

The walk-with-tool actions (`wrench_walk`, `tape_walk`) keep the tool in hand while walking: the tool arm holds the tool exactly as in the held pose for that direction (same `contact_pixels`), the legs stride and the free arm swings. They were appended after the original 128 frames, so earlier atlas indices are unchanged. The game cuts cleanly between walking, holding and using: no tool is ever used mid-walk.

## Rebuild and audit

From the repository root with the installed sprite-axi and the user's Aseprite binary configured:

```powershell
sprite-axi run assets/characters/engineer/source/build.lua
sprite-axi run assets/characters/engineer/source/preview.lua
node assets/characters/engineer/source/package-preview.cjs
node --preserve-symlinks --preserve-symlinks-main assets/characters/engineer/source/validate.cjs
```

**Pending rebuild:** the walk-with-tool frames were added by running this `build.lua` through a stand-in for the Aseprite API (no Aseprite on that machine). The PNGs, atlas, `manifest.json`, atlas JSON, `validation.json` and `preview-data.js` are current and pass `validate.cjs`. The original 128 frames are the committed originals, byte for byte. Still to refresh with real Aseprite: `engineer.aseprite` (still 128 frames) and `preview/overview.png` / `preview/motion.gif`. Run the four commands above once. The stand-in reproduced the original frames exactly, except 1–5 edge pixels on four 45° frames where platform `sin`/`cos` round differently.

The validator reuses the installed sprite-axi PNG decoder. Set `PNGJS_PATH` to another installed `pngjs` module if moving the source to a different machine. The optional local preview server runs with `node assets/characters/engineer/source/serve.cjs` and binds only `127.0.0.1:4173`.

Validation checks PNG counts, dimensions, binary alpha, palette membership, distinct poses/animation frames, fixed tool contacts, matching atlas pixels, animation tag ranges and slice pivot. Visual checks cover all directions at native size on both floor values. The game uses this pack through `src/art/engineer.rs` (see `src/art/README.md`); its unit tests check the atlas indices and tool contacts against `manifest.json`.
