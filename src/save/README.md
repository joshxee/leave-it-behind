# save

Saving settings and progress. A flight in progress is never saved. Both are loaded while the plugin builds and written in `Last` whenever either changes, so there is no save button and there are no slots. The frame that quits saves too.

- **Where:** on desktop, `settings.ron` and `save.ron` in `data_dir()`: `%APPDATA%\leave-it-behind` (Windows), `~/Library/Application Support/leave-it-behind` (macOS), `$XDG_DATA_HOME/leave-it-behind` or `~/.local/share/leave-it-behind` (Linux). Each is written to a `.tmp` file and renamed into place, so a crash mid-write leaves the old file. On the web, `localStorage` keys `leave-it-behind/settings` and `leave-it-behind/save`. Tests and native `e2e` builds keep saves in memory (`Storage::memory()`), so they never touch a real save.
- **Format:** RON with a `version` (`FORMAT_VERSION`). Missing fields take defaults and unknown fields are ignored, so adding a field needs no version bump. A file that cannot be read (corrupt, or saved by a newer game) is copied to `<key>.bak`, the game starts from defaults, and a notice says so. It never crashes.
- **Settings** include the coaching tips already shown (`tips_seen`, see `src/coach/README.md`), so the coaching plays once per player, not once per launch.
- **Progress:** `level::Progress`: per level id, flights, landings and the best `RunRecord` (see `src/level/README.md`).
- **Types and functions:** `Storage` (`memory`, `in_dir`, `disabled`, `platform_default`, `read`, `write`; insert one before `GamePlugin` to override the default), `data_dir`, `encode_settings` / `decode_settings`, `encode_progress` / `decode_progress`, `load`.
- **Systems:** `persist` (`Last`).
- **Dependencies:** `serde` and `ron`, both already in Bevy's tree. The folder lookup is our own: `dirs` and `directories` pull in `option-ext` (MPL-2.0), which `deny.toml` rejects.
- **Tests:** unit tests in `mod.rs`; integration tests in `tests/integration/save.rs`; e2e (settings survive a page reload) in `e2e/specs/menus.spec.ts`.
