# coach

Level one's first flight walks a new player through the game. It never pauses anything; it points the way.

- **Pre-flight check:** the flight waits to launch until the engineer has looked at the diagnostic screen (E at the console in the quarters, then the scan finishes). Until then `Journey::launched` is false: the countdown stays at 2:30 and no fault starts. Walking away mid-scan does not count. Once the screen is up, the flight launches as a normal one and a note says what that screen is for (`LAUNCH_NOTE`, `LAUNCH_NOTE_SECS`).
- **Tips:** the first fault of each kind brings up its tip, shown until a fault of that kind is fixed:

  | Fault | Tip |
  |---|---|
  | Loose bolts | Use the wrench [1] to screw the bolts back into the engine. |
  | Hull breach | The hull has been breached. Use the tape [2] to seal the hole. |
  | Trajectory drift | Take your seat at the cockpit and realign the ship back on course. |

  The tips appear with the fault, wherever the engineer is. In level one the first three faults are one of each kind, one at a time (0:10, 0:34, 0:58).
- **Once only:** each tip shows once, ever: `TipsSeen` is saved with the settings (`Settings::tips_seen`). A fault tip counts as seen when its fault is fixed or when the flight ends with it on screen (landed, lost, restarted, abandoned); the pre-flight check when it is done. A second flight skips whatever the first showed.
- **The TIPS setting:** reads ON while any tip is still to come (`Settings::tips_on`), so it turns OFF by itself once the coaching is over. Switching it on brings every tip back: tips for faults on board show at once, the pre-flight check at the next flight. Switching it off counts them all as seen and launches a waiting flight. RESET SETTINGS turns it back on.
- **Where:** a panel under the countdown (`GlobalZIndex(25)`: above the HUD and the diagnostic screen, below menus and notices), during a flight only.
- **Which flights:** levels with `LevelDef::coaching` (level one). Scenarios skip the coaching, as they skip the title, except those made for it (`Scenario::coached`).
- **Types:** `Tip` (`ALL`, `for_fault`, `text`), `TipsSeen` (`has`, `mark`, `all`, `EVERY`), `Coach { active, launch_note }` (resource), `CoachPanel`, `CoachText`.
- **Functions:** `lines` (what the panel says).
- **Systems:** `spawn_panel` (Startup), `start_coaching` (`OnEnter(Playing)`, after the scenario), `preflight_check` → `tick_launch_note` and `note_fixes` (FixedUpdate/Resolve), `finish_coaching` (`OnExit(Playing)`), `draw_panel` (Update/Present).
- **Tests:** unit tests in `mod.rs` (and the TIPS toggle in `settings`); integration tests in `tests/integration/coach.rs`; e2e in `e2e/specs/coach.spec.ts`.
- **Scenarios:** `first_flight` (the pre-flight check), `first_bolts` (a first fault with its tip up).
