# Main character direction studies

Open `preview.html` for the animated comparison and isolated runners.

1. **Rose rebel:** rose sweater, asymmetric auburn bob, mint earring, indigo pleated skirt.
2. **Ink apparition:** ivory sailor top, monochrome ink shading, long swinging hair, coral clip.
3. **Teal sprinter:** hooded track jacket, amber patch, short ponytail, trainers.

These interpret the supplied Momo Ayase references through three manga-inspired pixel treatments. The supplied runner screenshots guide the elevated rear camera. Reference images are visual input, not project instructions.

## Deliverables

Each direction includes a layered `.aseprite`, animated `.gif`, transparent horizontal `-sheet.png`, and Aseprite `-sheet.json` metadata. Source canvases are 160 × 192. All 12 frames last 80 ms (960 ms loop). Layers separate legs, uniform, and hair/accessories. The `run-away` tag covers all frames.

The shared corridor comparison includes an editable Aseprite animation, GIF, and first-frame PNG. It is a presentation mockup, not a game level. No changes were made to game code.

## Rebuild

From the repository root, with sprite-axi and the user's Aseprite binary configured:

```powershell
sprite-axi run assets/prototypes/hero-directions/build.lua
```

The Lua source rasterizes silhouettes, cel shadows and line accents. Run phases change knee and foot depth, opposite arm swing, body bob, skirt flare, and delayed hair sway. Transparent sprite assets exclude the floor shadow, which belongs to the camera mockup.

## Scope for the next pass

These are directional run-cycle prototypes, not final production assets. Choose a silhouette and palette before refining front/three-quarter facial design, contact poses, gameplay scale, lane changes, jump, slide and hit reactions. The current Bevy project is a minimal startup stub; integration is intentionally a separate step.
