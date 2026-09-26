# diagnostics

The diagnostic console in the engineer's quarters: a deliberate cost. E at the console scans for `SCAN_SECS` (1 s), then shows a minimap of the whole ship with a red marker on every loose bolt, breach and the helm, plus a list of active faults (room, spot, seconds left), most urgent first. It is the only place fault locations are shown. E again, or walking out of reach, closes it. Time keeps running while it is open.

- **Resources:** `Diagnostics { view: DiagView::{Closed, Scanning, Open}, uses }`.
- **Functions:** `readings` (what the screen shows), `reading_line`, `to_map` (world to minimap pixels).
- **Systems:** `spawn_console`, `spawn_overlay` (Startup), `reset` (`OnEnter(Playing)`), `use_console` (FixedUpdate/Act), `scan` (FixedUpdate/Simulate), `draw_overlay` (Update/Present; UI at `GlobalZIndex(20)`).
- **Tests:** unit tests in `mod.rs`; integration tests in `tests/integration/diagnostics.rs`; e2e in `e2e/specs/faults.spec.ts`.
- **Scenarios:** `diagnostics`.
