//! Top-level app state and the system-set ordering every feature plugs into.

use bevy::prelude::*;

#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppState {
    /// First frame(s). A future loading screen lives here.
    #[default]
    Boot,
    /// Main gameplay.
    Playing,
}

impl AppState {
    pub fn as_str(self) -> &'static str {
        match self {
            AppState::Boot => "Boot",
            AppState::Playing => "Playing",
        }
    }
}

/// Ordering for gameplay systems. Configured (chained) in `Startup`,
/// `Update` and `FixedUpdate`, so every feature slots into the same pipeline.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameSet {
    /// Spawning entities (Startup) / reading raw input (Update).
    Input,
    /// Applying scenarios at startup, simulation in `FixedUpdate`.
    Simulate,
    /// Reacting to simulation results (score, messages).
    Resolve,
    /// Presentation: UI text, audio.
    Present,
}

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>();
        for schedule in [Startup.intern(), Update.intern(), FixedUpdate.intern()] {
            app.configure_sets(
                schedule,
                (
                    GameSet::Input,
                    GameSet::Simulate,
                    GameSet::Resolve,
                    GameSet::Present,
                )
                    .chain(),
            );
        }
        app.add_systems(Update, finish_boot.run_if(in_state(AppState::Boot)));
    }
}

/// Nothing to load yet: go straight to gameplay.
fn finish_boot(mut next: ResMut<NextState<AppState>>) {
    next.set(AppState::Playing);
}
