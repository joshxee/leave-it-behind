// Hide the console window on Windows release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::prelude::*;
use leave_it_behind::{GamePlugin, primary_window};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(primary_window()),
                ..default()
            }),
            GamePlugin,
        ))
        .run();
}
