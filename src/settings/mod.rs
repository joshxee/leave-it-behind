//! What the player can change in the settings screen, and applying it.
//! Loaded and saved by `save`; read by the alarm, the nav display, the
//! diagnostic map, the HUD, the coaching and the pause logic.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::coach::TipsSeen;

/// Percentages move in steps of this much.
pub const PERCENT_STEP: u8 = 25;

#[derive(Resource, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Screen shake strength, percent.
    pub shake: u8,
    /// Alarm flashing, percent. At 0 the red tint stops pulsing (it still
    /// deepens with urgency) and the nav display stops blinking.
    pub flash: u8,
    /// Show the controls hint at the start of a flight.
    pub controls_hint: bool,
    /// Coaching tips already shown (`coach`). The TIPS setting reads ON
    /// while any is still to come: switching it on brings them all back,
    /// switching it off counts them all as seen.
    pub tips_seen: TipsSeen,
    /// Pause when the game window loses focus.
    pub pause_unfocused: bool,
    /// Desktop only: borderless fullscreen.
    pub fullscreen: bool,
    /// Desktop only.
    pub vsync: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            shake: 100,
            flash: 100,
            controls_hint: true,
            tips_seen: TipsSeen::default(),
            pause_unfocused: true,
            fullscreen: false,
            vsync: true,
        }
    }
}

/// One row of the settings screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SettingKey {
    Shake,
    Flash,
    ControlsHint,
    Tips,
    PauseUnfocused,
    Fullscreen,
    Vsync,
}

impl SettingKey {
    pub const ALL: [SettingKey; 7] = [
        SettingKey::Shake,
        SettingKey::Flash,
        SettingKey::ControlsHint,
        SettingKey::Tips,
        SettingKey::PauseUnfocused,
        SettingKey::Fullscreen,
        SettingKey::Vsync,
    ];

    pub fn label(self) -> &'static str {
        match self {
            SettingKey::Shake => "SCREEN SHAKE",
            SettingKey::Flash => "ALARM FLASHING",
            SettingKey::ControlsHint => "CONTROLS HINT",
            SettingKey::Tips => "TIPS",
            SettingKey::PauseUnfocused => "PAUSE WHEN UNFOCUSED",
            SettingKey::Fullscreen => "FULLSCREEN",
            SettingKey::Vsync => "VSYNC",
        }
    }

    /// Settings the platform can honour. The browser owns fullscreen (itch
    /// has its own button) and frame pacing.
    pub fn available(self, web: bool) -> bool {
        !(web && matches!(self, SettingKey::Fullscreen | SettingKey::Vsync))
    }
}

impl Settings {
    pub fn shake_scale(&self) -> f32 {
        f32::from(self.shake.min(100)) / 100.0
    }

    pub fn flash_scale(&self) -> f32 {
        f32::from(self.flash.min(100)) / 100.0
    }

    /// The TIPS setting: on while any coaching tip is still to come.
    pub fn tips_on(&self) -> bool {
        !self.tips_seen.all()
    }

    /// Moves a setting one step down (`up == false`) or up. Percentages stop
    /// at 0 and 100; on/off settings flip either way.
    pub fn step(&mut self, key: SettingKey, up: bool) {
        let percent = |v: &mut u8| {
            *v = if up {
                v.saturating_add(PERCENT_STEP).min(100)
            } else {
                v.saturating_sub(PERCENT_STEP)
            }
        };
        match key {
            SettingKey::Shake => percent(&mut self.shake),
            SettingKey::Flash => percent(&mut self.flash),
            _ => self.toggle(key),
        }
    }

    /// Next value, wrapping (Enter or a click on the row).
    pub fn cycle(&mut self, key: SettingKey) {
        let wrap = |v: &mut u8| {
            *v = if *v >= 100 {
                0
            } else {
                (*v + PERCENT_STEP).min(100)
            }
        };
        match key {
            SettingKey::Shake => wrap(&mut self.shake),
            SettingKey::Flash => wrap(&mut self.flash),
            _ => self.toggle(key),
        }
    }

    fn toggle(&mut self, key: SettingKey) {
        let flag = match key {
            SettingKey::ControlsHint => &mut self.controls_hint,
            SettingKey::Tips => {
                self.tips_seen = if self.tips_on() {
                    TipsSeen::EVERY
                } else {
                    TipsSeen::default()
                };
                return;
            }
            SettingKey::PauseUnfocused => &mut self.pause_unfocused,
            SettingKey::Fullscreen => &mut self.fullscreen,
            SettingKey::Vsync => &mut self.vsync,
            SettingKey::Shake | SettingKey::Flash => return,
        };
        *flag = !*flag;
    }

    /// The value as the settings screen shows it.
    pub fn value_label(&self, key: SettingKey) -> String {
        let on_off = |b: bool| if b { "ON" } else { "OFF" }.to_string();
        match key {
            SettingKey::Shake => format!("{}%", self.shake),
            SettingKey::Flash => format!("{}%", self.flash),
            SettingKey::ControlsHint => on_off(self.controls_hint),
            SettingKey::Tips => on_off(self.tips_on()),
            SettingKey::PauseUnfocused => on_off(self.pause_unfocused),
            SettingKey::Fullscreen => on_off(self.fullscreen),
            SettingKey::Vsync => on_off(self.vsync),
        }
    }
}

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Settings>();
        #[cfg(not(target_arch = "wasm32"))]
        app.add_systems(
            PostUpdate,
            apply_to_window.run_if(resource_changed::<Settings>),
        );
    }
}

/// Fullscreen and vsync, on desktop.
#[cfg(not(target_arch = "wasm32"))]
fn apply_to_window(
    settings: Res<Settings>,
    mut windows: Query<&mut Window, With<bevy::window::PrimaryWindow>>,
) {
    use bevy::window::{MonitorSelection, PresentMode, WindowMode};
    for mut window in &mut windows {
        let mode = if settings.fullscreen {
            WindowMode::BorderlessFullscreen(MonitorSelection::Current)
        } else {
            WindowMode::Windowed
        };
        if window.mode != mode {
            window.mode = mode;
        }
        let present = if settings.vsync {
            PresentMode::AutoVsync
        } else {
            PresentMode::AutoNoVsync
        };
        if window.present_mode != present {
            window.present_mode = present;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percentages_step_and_stop_at_the_ends() {
        let mut s = Settings::default();
        s.step(SettingKey::Shake, true);
        assert_eq!(s.shake, 100);
        s.step(SettingKey::Shake, false);
        assert_eq!(s.shake, 75);
        for _ in 0..6 {
            s.step(SettingKey::Shake, false);
        }
        assert_eq!(s.shake, 0);
        assert_eq!(s.value_label(SettingKey::Shake), "0%");
        assert_eq!(s.shake_scale(), 0.0);
    }

    #[test]
    fn cycling_wraps_and_toggles_flip() {
        let mut s = Settings::default();
        s.cycle(SettingKey::Flash);
        assert_eq!(s.flash, 0);
        s.cycle(SettingKey::Flash);
        assert_eq!(s.flash, 25);
        s.cycle(SettingKey::ControlsHint);
        assert!(!s.controls_hint);
        s.step(SettingKey::ControlsHint, true);
        assert!(s.controls_hint);
        assert_eq!(s.value_label(SettingKey::ControlsHint), "ON");
    }

    #[test]
    fn tips_stay_on_until_every_tip_is_seen() {
        use crate::coach::Tip;
        let mut s = Settings::default();
        assert!(s.tips_on());
        s.tips_seen.mark(Tip::Preflight);
        s.tips_seen.mark(Tip::LooseBolts);
        assert!(s.tips_on(), "two tips are still to come");
        // Off counts every tip as seen; on brings them all back.
        s.cycle(SettingKey::Tips);
        assert_eq!(s.tips_seen, TipsSeen::EVERY);
        assert_eq!(s.value_label(SettingKey::Tips), "OFF");
        s.step(SettingKey::Tips, false);
        assert_eq!(s.tips_seen, TipsSeen::default());
        assert_eq!(s.value_label(SettingKey::Tips), "ON");
    }

    #[test]
    fn the_browser_owns_fullscreen_and_vsync() {
        let web: Vec<_> = SettingKey::ALL
            .into_iter()
            .filter(|k| k.available(true))
            .collect();
        assert!(!web.contains(&SettingKey::Fullscreen));
        assert!(!web.contains(&SettingKey::Vsync));
        assert!(SettingKey::ALL.into_iter().all(|k| k.available(false)));
    }
}
