# diagnostics

The diagnostic console in the engineer's quarters (the map's `'d'`, drawn with the pack's diagnostic screen: dark until used, a live waveform while scanning or open): a deliberate cost. E at its front edge scans for `SCAN_SECS` (1 s), then shows a minimap of the whole ship (fitted to the ship's shape, whichever way it points) with a red marker on every loose bolt, breach and the helm, plus a list of active faults (room, spot, seconds left), most urgent first. It is the only place fault locations are shown. E again, or walking out of reach, closes it. On a player's first flight, level one waits to launch until this screen has been looked at (`coach`). Time keeps running while it is open. (Its markers pulse less, down to steady, with the alarm flashing setting.)

- **Resources:** `Diagnostics { view: DiagView::{Closed, Scanning, Open}, uses }`.
- **Functions:** `readings` (what the screen shows), `reading_line`, `to_map` (world to minimap pixels), `map_size`.
- **Systems:** `spawn_console`, `spawn_overlay` (Startup), `reset` (`OnEnter(Playing)`), `use_console` (FixedUpdate/Act), `scan` (FixedUpdate/Simulate), `draw_overlay` (UI at `GlobalZIndex(20)`), `light_console` (Update/Present).
- **Tests:** unit tests in `mod.rs`; integration tests in `tests/integration/diagnostics.rs`; e2e in `e2e/specs/faults.spec.ts`.
- **Scenarios:** `diagnostics`.
