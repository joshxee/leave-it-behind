# settings

What the player can change, and applying it. `Settings` is loaded and saved by `save`; the settings screen (`menu`) changes it with `step` (Left/Right, `<` `>`) and `cycle` (Enter, click).

| Setting | Values | Effect |
|---|---|---|
| Screen shake | 0-100%, steps of 25 | scales the alarm's camera shake (`alarm`) |
| Alarm flashing | 0-100%, steps of 25 | scales the alarm tint's pulse, the nav display's blink and the diagnostic markers' pulse. At 0 they hold steady; the tint still deepens with urgency, so no warning is lost |
| Controls hint | on / off | the controls line at the start of a flight (`ui`) |
| Pause when unfocused | on / off | pause when the window loses focus (`menu`) |
| Fullscreen | on / off | desktop only: borderless fullscreen |
| Vsync | on / off | desktop only: `PresentMode::AutoVsync` / `AutoNoVsync` |

The web build hides fullscreen and vsync (`SettingKey::available`): the browser owns both, and itch has its own fullscreen button.

- **Types:** `Settings` (`shake_scale`, `flash_scale`, `step`, `cycle`, `value_label`), `SettingKey` (`ALL`, `label`, `available`).
- **Systems:** `apply_to_window` (PostUpdate, desktop only, when `Settings` changes).
- **Tests:** unit tests in `mod.rs`; integration tests in `tests/integration/menu.rs` and `tests/integration/save.rs`; e2e in `e2e/specs/menus.spec.ts`.
- **Scenarios:** `settings`.
