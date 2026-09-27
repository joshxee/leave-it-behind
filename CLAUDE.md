# Leave It Behind: agent index

> **Hard limit: 150 lines** (`scripts/test-all.sh` and CI fail above it). This file is an
> index, not a manual. When adding guidance, put the detail in `TESTING.md` or the relevant
> feature's `src/<feature>/README.md`, and add at most one line here that points to it.

## Bevy version

- Pinned: **bevy 0.19.1** (`Cargo.toml`). APIs change every minor version (0.17 renamed
  events to messages: `add_message`, `MessageReader/Writer`, `Messages<T>`).
- Check every API against 0.19 docs/examples or `~/.cargo/registry/src/*/bevy_*-0.19.1/`.
  Never write Bevy code from memory.

## Architecture

- `src/main.rs` only adds `DefaultPlugins` + `GamePlugin`. All game code is in the library (`src/lib.rs`).
- The game: a five-level campaign of a top-down ship-repair time-management game (`level/campaign.rs`). Design and tuning
  live in the feature READMEs (start with `src/level/README.md` and `src/faults/README.md`).
- The ship is data: the ASCII map `SHIP` in `src/ship/map.rs` (legend and rules in its docs,
  `src/ship/README.md`). Rooms, walls, doors, colliders and fault sites are all derived from it.
- One plugin per feature module, each with a `README.md`: `ship/`, `player/`, `tools/`,
  `faults/`, `level/`, `diagnostics/`, `alarm/`, `ui/`, `menu/`, `settings/`, `save/`, `scenarios/`,
  `coach/` (level one's first-flight coaching: pre-flight check, one tip per fault kind, TIPS setting),
  `upgrades/` (the pick of three between levels: run faster, faster wrench, wider tape).
- Flow: `Boot` → `Menu` (title, main menu; `src/menu/README.md`) → `Playing` → `Landed`/`Lost`;
  a landing continues through the upgrade screen to the next level.
  Pausing is the `Pause` sub-state of `Playing`, so it never restarts the run.
- Infrastructure: `state.rs` (`AppState`, `Pause`, `running`/`not_paused`, `GameSet` order, `RunSet`,
  `RunEntity`), `rng.rs` (`GameRng`),
  `art/` (atlases, tile and engineer data from the packs' manifests: `src/art/README.md`),
  `shapes.rs` (placeholder circle/ring sprites), `palette.rs`, `determinism.rs`
  (`TestDeterminismPlugin`), `version.rs`, `e2e_bridge.rs` (wasm + `e2e` only).
- **New feature plugin:** create `src/<feature>/mod.rs` with `pub struct <Feature>Plugin`,
  add `pub mod <feature>;` in `lib.rs`, add the plugin to the tuple in
  `GamePlugin::build`, put systems in a `GameSet` (gameplay in `FixedUpdate`,
  `.run_if(running)`; in-world animation `.run_if(not_paused)`), and write `src/<feature>/README.md`
  (purpose, components/resources, test module, scenarios).
- Release process: `docs/RELEASING.md`.

## Commands

```bash
cargo build                          # native build
cargo run --features dev             # run natively (dynamic linking, local only)
cargo test                           # unit + integration (headless)
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all
E2E=1 scripts/build-web.sh           # web build with the test bridge (-> wasm/)
scripts/build-web.sh --release       # player web build: size budget + leak check
scripts/serve-web.sh                 # serve wasm/ on http://localhost:4173
scripts/smoke-native.sh              # headless native screenshot
(cd e2e && npx playwright test)      # browser e2e (needs an E2E web build)
scripts/test-all.sh [--scope all|rust|native|web]   # everything -> test-reports/latest/REPORT.md
```

## Skills and agents

- `/run`, `/verify`: follow `.claude/skills/run-game/SKILL.md`. `/verify` is overridden
  by `.claude/skills/verify/SKILL.md`. Regenerate the run recipe with
  `/run-skill-generator` if the build or launch process changes.
- `/test-report`: full suite on a cheap model (`test-runner` agent). Returns only the REPORT path + STATUS.
- `/finish-feature`: end-of-feature loop (tests → /verify → /test-report → fix, max 3 loops).

## Definition of done (every feature)

1. Unit tests and ECS integration tests added (`TESTING.md`).
2. Playwright spec added or updated if the change is visible or interactive.
3. `/verify` run and its screenshots inspected.
4. `/test-report` run and `REPORT.md` reviewed (`STATUS: PASS`).
5. fmt and clippy clean.

## Rules

- Gameplay runs in `FixedUpdate`, reading input via `PlayerIntent`-style resources written in `Update`.
- A run starts in `OnEnter(AppState::Playing)` (launch and every R restart): tag per-run entities
  `RunEntity` and reset per-run resources in `RunSet::Spawn`.
- Art comes from the packs under `assets/` through `art/` (1 art pixel = 1 world unit, nearest
  sampling). Things with no art yet are shapes in the pack palette, listed in `docs/ART_REQUESTS.md`.
- Colours: decoration uses the pack's cold palette; warm colours are only for danger (a test
  in `palette.rs` enforces both). Add colours to `palette.rs`, never inline.
- Never hard-code a position, direction or room size: derive it from the ship map (the layout
  and the way the ship points will change). Name sites ship-relative (port/fore), not by screen.
- The engineer always holds a tool; tools work only standing still (walking stops any use).
  Animations cut cleanly: no blending between walk, hold and use.
- Builds ship only the files in `assets/RUNTIME_ASSETS` (`scripts/stage-assets.sh`). List every
  file the game loads there; the rest of `assets/` is source art and never ships.
- Art packs are generated by their `source/build.lua` (sprite-axi + Aseprite). Change art there,
  then run the pack's `validate.cjs` (see the pack README), never by editing PNGs.
- UI text is ASCII only (the bundled font has no other glyphs).
- Randomness only through `ResMut<GameRng>`. Never `rand::rng()` / `thread_rng()`.
- Never add a top-level file under `tests/`. Add a module to `tests/integration/main.rs` (`TESTING.md`).
- Every new game situation gets a scenario in `src/scenarios/`, listed in the feature's `README.md`.
- Only settings and progress are saved (`src/save/README.md`). Tests keep saves in memory; never
  point a test at a real save folder.
- `ci/budgets.env` (`WASM_BUDGET_KB`) is a reviewed limit. Never raise it to pass a check.
- Screenshot baselines are Linux-only and approved by a human or the primary model (`TESTING.md`).
- The `e2e` test bridge must never ship. `build-web.sh` fails non-e2e builds containing it.
- Hide non-deterministic UI (FPS, timestamps, version text) under `e2e`.
- `dev` (dynamic linking) is for local native builds only. Never enable it in CI, release, or wasm.

## Gotchas

- Call `app.update()` once before asserting on anything `Startup` creates.
- Door leaves move, so they are not in `Colliders`: anything that moves the player adds the
  closed ones (`Door::leaf`), as `player::move_player` does.
- `Anchor` (sprite pivot) is `bevy::sprite::Anchor`, not in the prelude. The engineer's pivot is
  its feet (`art::engineer::ANCHOR`): its transform is the ground point, not the sprite centre.
- Players boot to the title; any scenario (and the test helper `boot`) skips it. Use `boot_to_menu`
  natively, or `openGame(page)` without a scenario on the web, to start at the title.
- `MinimalPlugins` has no window, input, assets, or rendering: `test_app()` adds input
  resources and a fake `Window`. Asset-dependent plugins skip themselves without `AssetServer`.
- In a frame, `FixedUpdate` runs before `Update`, so input read in `Update` applies next frame.
- wasm is single-threaded: no blocking, no `std::thread`, no `std::time::Instant` (use `bevy::platform::time`).
- Browsers block audio until the user interacts with the page; the title menu provides that interaction before the flight's sound cues.
- Aiming converts the cursor with the room camera's `anchor`: after teleporting the player in a
  test, run a frame before `aim_at` so the camera has cut to the new room.
- Don't enable `dynamic_linking` on wasm.
- `wasm-bindgen-cli` must exactly match the `wasm-bindgen` crate version in `Cargo.lock`.
- Assets load relative to the executable unless `BEVY_ASSET_ROOT` is set (as `smoke-native.sh` does).

## Bevy features

Default features plus `wav` and, on wasm, `web`. Nothing trimmed yet. If the wasm budget is
exceeded, drop 3d (pbr, gltf) and picking first, and list the dropped features here.
