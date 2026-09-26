# Ship depth study

A review-only room mockup. The existing 64-tile ship pack is unchanged.

Open `index.html` to compare the original treatment with the raised version and toggle full-height/cutaway foreground walls. All three displayed images are embedded in the HTML, so the comparison and toggles also work offline without a preview server. The comparison uses the same floor layout and engineer ground position throughout.

- `current.png`: original flat tiles.
- `raised-solid.png`: raised walls with correct foreground occlusion.
- `raised-cutaway.png`: the same depth treatment with a low foreground wall for readability.
- `comparison.png`: original on the left; raised/cutaway on the right.
- `depth-study.aseprite`: three editable indexed frames with named tags.

The wall cap rises 22 screen pixels above its footprint. The optional near-wall cutaway is 6 pixels high. All views use the existing engineer palette and keep the engineer ground pivot at the same location. Taller dark faces, recessed door framing and a sloped diagnostic console supply the depth cues. Sidewalls show mainly their cap surfaces; the corner joins and broken ends reveal thickness. The side breach is widened to keep its opening visible behind the elevated near stump.

This is a composed visual study, not a new export of modular game tiles. The full tileset, collision data, animations and Bevy rendering have not been revised. The next production pass can split the selected treatment into base/foreground art where needed and retain the ground footprints.

Rebuild from the repository root:

```powershell
sprite-axi run assets/environment/derelict-ship-depth-study/source/build.lua
node assets/environment/derelict-ship-depth-study/source/package-preview.cjs
```

Edit `source/preview-template.html` for page changes, then rerun the packaging command. Links to the PNG downloads and editable Aseprite file use the adjacent files.
