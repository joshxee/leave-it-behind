//! Leave It Behind: game library.
//!
//! `main.rs` only adds [`DefaultPlugins`] and [`GamePlugin`]. Everything else
//! lives here so integration tests in `tests/integration/` can import it.

use bevy::prelude::*;
use bevy::window::WindowResolution;

pub mod audio;
pub mod determinism;
#[cfg(all(feature = "e2e", target_arch = "wasm32"))]
mod e2e_bridge;
pub mod player;
pub mod rng;
pub mod scenarios;
pub mod scoring;
pub mod state;
pub mod ui;
pub mod version;

pub use determinism::TestDeterminismPlugin;
pub use rng::GameRng;
pub use scenarios::{ActiveScenario, Scenario};
pub use state::{AppState, GameSet};

/// Canvas and window size in logical pixels. Also the Playwright viewport.
pub const WINDOW_SIZE: UVec2 = UVec2::new(1280, 720);
/// Gameplay tick rate. Gameplay systems run in `FixedUpdate`.
pub const FIXED_HZ: f64 = 60.0;

/// The whole game. Made of one plugin per feature module.
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Time::<Fixed>::from_hz(FIXED_HZ))
            .add_plugins((
                state::StatePlugin,
                rng::RngPlugin,
                player::PlayerPlugin,
                scoring::ScoringPlugin,
                ui::UiPlugin,
                audio::AudioFxPlugin,
                scenarios::ScenarioPlugin,
                version::VersionPlugin,
            ));

        #[cfg(feature = "e2e")]
        app.add_plugins(TestDeterminismPlugin);
        #[cfg(all(feature = "e2e", target_arch = "wasm32"))]
        app.add_plugins(e2e_bridge::E2eBridgePlugin);
    }
}

/// Primary window settings shared by the binary and any headless harness.
pub fn primary_window() -> Window {
    Window {
        title: "Leave It Behind".into(),
        resolution: WindowResolution::new(WINDOW_SIZE.x, WINDOW_SIZE.y),
        canvas: Some("#bevy-canvas".into()),
        // Under e2e the canvas keeps a fixed size so screenshots are stable.
        fit_canvas_to_parent: !cfg!(feature = "e2e"),
        prevent_default_event_handling: true,
        ..default()
    }
}
