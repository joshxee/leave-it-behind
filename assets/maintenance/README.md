# Maintenance art

Editable indexed Aseprite assets using the existing engineer palette, binary transparency and nearest sampling. `index.html` is the animated review, with an actual game render of the revised foreground wall.

| Sheet | Cell | Frames | Use |
|---|---|---|---|
| radar | 128 × 144 | 1 | CRT housing; screen centre (64, 64) |
| radar-sweep | 104 × 104 | 24 | Sweep behind the target |
| nav-ship | 16 × 20 | 2 | Course cursor and thruster glint |
| helm-base / helm-stick | 32 × 32 / 12 × 18 | 1 each | Control pad and tilting stick |
| engine / engine-vertical | 224 × 88 / 128 × 400 | 4 each | Horizontal and vertical map footprints |
| bolt | 32 × 32 | 9 | Exposed thread to seated hex; follows the 0.9 s wrench turn |
| tape-strip | 32 × 12 | 6 | 70 ms application frames, then hold the final frame |
| tape-roll / wrench | 24 × 24 / 40 × 16 | 4 / 1 | Standalone art; the game retains the latest engineer's integrated tool poses |
| breach | 40 × 32 | 1 | Torn metal on the raised wall face |
| wall-depth | 64 × 88 | 256 | 16 joining masks × 16 room-facing quadrant masks |
| door-depth | 64 × 88 | 5 | Four sliding-door frames and a locked hatch |

Sheets are untrimmed, without gutters, with up to 32 columns. `manifest.json` records dimensions, column counts and timing. Wall variants are layout frames, not an animation.

All walls keep a **22 px raised face** and **12 px cap**, shifted **18 px outboard** within the grid cells. There is no low foreground variant. The wall tile's ground pivot is (32, 56). The map's collision core remains a guard band that keeps the engineer visible; door interactions and passage geometry are unchanged. The camera includes additional margin for the raised silhouette and HUD.

Bolts keep their socket at (16, 22), expose ten pixels of thread, and turn clockwise while retracting. A cancelled turn restores the exposed frame; repaired bolts stay seated. Tape feeds from the current engineer pose onto the raised surface, unfolds into woven strips, and persists after sealing until the next run. Holes and tape share `ship::depth::wall_art_point`.

`src/art/maintenance.rs` owns sheet handles and frame selection. The existing character and environment atlases remain in use. Only sheets listed in `assets/RUNTIME_ASSETS` ship in builds; source files, individual frames and the preview stay out of runtime builds.

Rebuild from the repository root:

```powershell
sprite-axi run assets/maintenance/source/build.lua
node --preserve-symlinks --preserve-symlinks-main assets/maintenance/source/validate.cjs
cargo test --no-default-features
```

`cargo run --example maintenance_smoke -- quiet` captures the engineer at the foreground wall. `bolts`, `breach`, and `drift` exercise the real repairs and assert completion. The example uses in-memory saves and writes screenshots under `test-reports/maintenance/`; on Windows set `BEVY_ASSET_ROOT` to the repository directory. No asset-generation tools are needed to play.
