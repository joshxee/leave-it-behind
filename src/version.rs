//! Build version (set by `build.rs`), logged at startup and shown as small
//! corner text. The corner text is hidden under `e2e` so it never changes
//! screenshot baselines.

use bevy::prelude::*;

pub const GAME_VERSION: &str = env!("GAME_VERSION");

#[derive(Component, Debug)]
pub struct VersionText;

pub struct VersionPlugin;

impl Plugin for VersionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, log_version);
        #[cfg(not(feature = "e2e"))]
        app.add_systems(Startup, spawn_version_text);
    }
}

fn log_version() {
    info!("Leave It Behind version {GAME_VERSION}");
}

#[cfg(not(feature = "e2e"))]
fn spawn_version_text(mut commands: Commands) {
    commands.spawn((
        VersionText,
        Text::new(GAME_VERSION),
        TextFont::from_font_size(12.0),
        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.5)),
        Node {
            position_type: PositionType::Absolute,
            bottom: px(4),
            right: px(6),
            ..default()
        },
        // Above the menus (40) and notices (60): players read it off any screen.
        GlobalZIndex(70),
    ));
}
