//! Score text (top-left). Uses the bundled Super Indie font when an
//! `AssetServer` exists; headless tests fall back to the default font.

use bevy::prelude::*;

use crate::GameSet;
use crate::scoring::Score;

pub const FONT_PATH: &str = "fonts/super-indie-font/SuperIndie-GOp7O.ttf";

#[derive(Component, Debug)]
pub struct ScoreText;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_score_text.in_set(GameSet::Input))
            .add_systems(
                Update,
                update_score_text
                    .in_set(GameSet::Present)
                    .run_if(resource_changed::<Score>),
            );
    }
}

pub fn score_label(score: u32) -> String {
    format!("Score: {score}")
}

/// The game font, or the default font when there is no asset server.
pub fn game_font(asset_server: Option<&AssetServer>, size: f32) -> TextFont {
    let mut font = TextFont::from_font_size(size);
    if let Some(server) = asset_server {
        font.font = server.load(FONT_PATH).into();
    }
    font
}

fn spawn_score_text(mut commands: Commands, asset_server: Option<Res<AssetServer>>) {
    commands.spawn((
        ScoreText,
        Text::new(score_label(0)),
        game_font(asset_server.as_deref(), 40.0),
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: px(16),
            left: px(16),
            ..default()
        },
    ));
}

fn update_score_text(score: Res<Score>, mut texts: Query<&mut Text, With<ScoreText>>) {
    for mut text in &mut texts {
        text.0 = score_label(score.0);
    }
}
