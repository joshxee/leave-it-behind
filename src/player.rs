//! The player: a colored square moved with the arrow keys.

use bevy::prelude::*;

use crate::{AppState, GameRng, GameSet, WINDOW_SIZE};

/// Pixels per second.
pub const PLAYER_SPEED: f32 = 300.0;
pub const PLAYER_SIZE: f32 = 48.0;

#[derive(Component, Debug)]
pub struct Player;

/// What the player asked for this frame. Written in `Update` from raw input,
/// consumed by fixed-step gameplay so input is never lost or doubled.
#[derive(Resource, Default, Debug)]
pub struct PlayerIntent {
    /// Held direction (not normalized).
    pub direction: Vec2,
    /// Number of Space presses not yet consumed by scoring.
    pub score_presses: u32,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerIntent>()
            .add_systems(Startup, (spawn_camera, spawn_player).in_set(GameSet::Input))
            .add_systems(
                Update,
                read_input
                    .in_set(GameSet::Input)
                    .run_if(in_state(AppState::Playing)),
            )
            .add_systems(
                FixedUpdate,
                move_player
                    .in_set(GameSet::Simulate)
                    .run_if(in_state(AppState::Playing)),
            );
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn spawn_player(mut commands: Commands, mut rng: ResMut<GameRng>) {
    let color = Color::hsl(rng.unit() * 360.0, 0.7, 0.55);
    commands.spawn((
        Player,
        Sprite::from_color(color, Vec2::splat(PLAYER_SIZE)),
        Transform::default(),
    ));
}

/// Maps held arrow keys to a direction vector.
pub fn direction_from_keys(keys: &ButtonInput<KeyCode>) -> Vec2 {
    let mut dir = Vec2::ZERO;
    if keys.pressed(KeyCode::ArrowLeft) {
        dir.x -= 1.0;
    }
    if keys.pressed(KeyCode::ArrowRight) {
        dir.x += 1.0;
    }
    if keys.pressed(KeyCode::ArrowUp) {
        dir.y += 1.0;
    }
    if keys.pressed(KeyCode::ArrowDown) {
        dir.y -= 1.0;
    }
    dir
}

fn read_input(keys: Res<ButtonInput<KeyCode>>, mut intent: ResMut<PlayerIntent>) {
    intent.direction = direction_from_keys(&keys);
    if keys.just_pressed(KeyCode::Space) {
        intent.score_presses += 1;
    }
}

/// New position after moving `direction` for `dt` seconds, kept inside the arena.
pub fn step_position(pos: Vec2, direction: Vec2, dt: f32) -> Vec2 {
    let half = WINDOW_SIZE.as_vec2() / 2.0 - Vec2::splat(PLAYER_SIZE / 2.0);
    (pos + direction.normalize_or_zero() * PLAYER_SPEED * dt).clamp(-half, half)
}

fn move_player(
    time: Res<Time>,
    intent: Res<PlayerIntent>,
    mut players: Query<&mut Transform, With<Player>>,
) {
    for mut transform in &mut players {
        let next = step_position(
            transform.translation.truncate(),
            intent.direction,
            time.delta_secs(),
        );
        transform.translation = next.extend(transform.translation.z);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagonal_movement_is_not_faster() {
        let p = step_position(Vec2::ZERO, Vec2::new(1.0, 1.0), 1.0 / 60.0);
        assert!((p.length() - PLAYER_SPEED / 60.0).abs() < 1e-3);
    }

    #[test]
    fn position_is_clamped_to_arena() {
        let p = step_position(Vec2::new(10_000.0, 0.0), Vec2::X, 1.0);
        assert_eq!(p.x, WINDOW_SIZE.x as f32 / 2.0 - PLAYER_SIZE / 2.0);
    }

    #[test]
    fn opposite_keys_cancel() {
        let mut keys = ButtonInput::<KeyCode>::default();
        keys.press(KeyCode::ArrowLeft);
        keys.press(KeyCode::ArrowRight);
        assert_eq!(direction_from_keys(&keys), Vec2::ZERO);
    }
}
