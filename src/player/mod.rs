//! The engineer: WASD movement that slides along walls (and is steered into
//! doorways), facing toward the mouse, and E for whichever console is in
//! reach. The sprite and its animations are in `sprite.rs`.

pub mod sprite;

use bevy::input::mouse::{AccumulatedMouseScroll, MouseScrollUnit};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::art::Art;
use crate::shapes::at;
use crate::ship::doors::{Door, door_assist};
use crate::ship::layout::{self, move_circle};
use crate::ship::{CameraRig, Colliders, cursor_to_world};
use crate::upgrades::Upgrades;
use crate::{AppState, GameSet, RunEntity, RunSet, running};

/// World units per second (before the run-faster upgrade).
pub const PLAYER_SPEED: f32 = 280.0;
/// Radius of the engineer's footprint (under 16, so a door's 36-unit gap fits).
pub const PLAYER_RADIUS: f32 = 14.0;
/// Pixel-unit scroll deltas (browsers, touchpads) per tool step.
const SCROLL_PIXELS_PER_STEP: f32 = 60.0;

#[derive(Component, Debug)]
pub struct Player;

/// Unit vector the player faces (toward the mouse). Tools point this way.
#[derive(Component, Debug, Clone, Copy)]
pub struct Facing(pub Vec2);

/// Present while the player is strapped in (at the helm): no walking.
#[derive(Component, Debug)]
pub struct Locked;

/// Whether the engineer is walking (WASD held and free to move), and the
/// last direction they walked. Tools only work while not walking.
#[derive(Component, Debug, Default, Clone, Copy, PartialEq)]
pub struct Movement {
    pub walking: bool,
    pub heading: Vec2,
}

/// What the player asked for. Written in `Update` from raw input, consumed
/// by fixed-step gameplay so presses are never lost or doubled.
#[derive(Resource, Default, Debug, Clone)]
pub struct PlayerIntent {
    /// Held WASD direction (not normalized).
    pub direction: Vec2,
    /// World point under the mouse cursor, if it is over the window.
    pub aim: Option<Vec2>,
    /// Left mouse button held.
    pub use_held: bool,
    /// Left clicks not yet consumed.
    pub use_presses: u32,
    /// E presses not yet consumed.
    pub interact_presses: u32,
    /// Belt slot picked with a number key, not yet consumed.
    pub select: Option<usize>,
    /// Scroll-wheel steps not yet consumed (positive = next tool).
    pub cycle: i32,
}

/// Something E works on.
#[derive(Component, Debug, Clone, Copy)]
pub struct Interactable {
    pub kind: InteractKind,
    /// Reach from the player's center to this entity's position.
    pub range: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractKind {
    Helm,
    Diagnostics,
}

/// The console within reach, if any. Drives the HUD prompt.
#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Focus(pub Option<InteractKind>);

/// Sent once per tick with an E press; `target` is the console in reach.
#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct InteractPressed {
    pub target: Option<InteractKind>,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerIntent>()
            .init_resource::<Focus>()
            .add_message::<InteractPressed>()
            .add_systems(
                OnEnter(AppState::Playing),
                spawn_player.in_set(RunSet::Spawn),
            )
            .add_systems(Update, read_input.in_set(GameSet::Input).run_if(running))
            .add_systems(
                FixedUpdate,
                (move_player, face_aim, update_focus, send_interact)
                    .chain()
                    .in_set(GameSet::Move)
                    .run_if(running),
            )
            .add_plugins(sprite::EngineerSpritePlugin);
    }
}

fn spawn_player(mut commands: Commands, art: Res<Art>, mut intent: ResMut<PlayerIntent>) {
    *intent = PlayerIntent::default();
    commands.spawn((
        Player,
        RunEntity,
        Name::new("Player"),
        Facing(Vec2::X),
        Movement {
            walking: false,
            heading: Vec2::NEG_Y,
        },
        at(layout::player_spawn(), 3.0),
        Visibility::default(),
        children![sprite::engineer_sprite(&art)],
    ));
}

/// Maps held WASD keys to a direction vector.
pub fn direction_from_keys(keys: &ButtonInput<KeyCode>) -> Vec2 {
    let mut dir = Vec2::ZERO;
    if keys.pressed(KeyCode::KeyA) {
        dir.x -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) {
        dir.x += 1.0;
    }
    if keys.pressed(KeyCode::KeyW) {
        dir.y += 1.0;
    }
    if keys.pressed(KeyCode::KeyS) {
        dir.y -= 1.0;
    }
    dir
}

const SLOT_KEYS: [(KeyCode, KeyCode); 9] = [
    (KeyCode::Digit1, KeyCode::Numpad1),
    (KeyCode::Digit2, KeyCode::Numpad2),
    (KeyCode::Digit3, KeyCode::Numpad3),
    (KeyCode::Digit4, KeyCode::Numpad4),
    (KeyCode::Digit5, KeyCode::Numpad5),
    (KeyCode::Digit6, KeyCode::Numpad6),
    (KeyCode::Digit7, KeyCode::Numpad7),
    (KeyCode::Digit8, KeyCode::Numpad8),
    (KeyCode::Digit9, KeyCode::Numpad9),
];

/// Belt slot (0-based) for a number key pressed this frame.
pub fn slot_from_keys(keys: &ButtonInput<KeyCode>) -> Option<usize> {
    SLOT_KEYS
        .iter()
        .position(|&(digit, numpad)| keys.just_pressed(digit) || keys.just_pressed(numpad))
}

/// Converts one frame of scroll input to at most one tool step, keeping
/// partial pixel scrolling in `carry`. Scrolling down (negative y) moves to
/// the next tool.
pub fn scroll_steps(delta_y: f32, unit: MouseScrollUnit, carry: &mut f32) -> i32 {
    if delta_y == 0.0 {
        *carry = 0.0;
        return 0;
    }
    match unit {
        // Some browsers report several lines per notch: still one step.
        MouseScrollUnit::Line => {
            *carry = 0.0;
            -delta_y.signum() as i32
        }
        MouseScrollUnit::Pixel => {
            *carry -= delta_y / SCROLL_PIXELS_PER_STEP;
            let steps = carry.trunc().clamp(-1.0, 1.0);
            *carry -= steps;
            steps as i32
        }
    }
}

fn read_input(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    scroll: Option<Res<AccumulatedMouseScroll>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    rigs: Query<&CameraRig>,
    mut intent: ResMut<PlayerIntent>,
    mut scroll_carry: Local<f32>,
) {
    intent.direction = direction_from_keys(&keys);
    intent.use_held = mouse.pressed(MouseButton::Left);
    if mouse.just_pressed(MouseButton::Left) {
        intent.use_presses += 1;
    }
    if keys.just_pressed(KeyCode::KeyE) {
        intent.interact_presses += 1;
    }
    if let Some(slot) = slot_from_keys(&keys) {
        intent.select = Some(slot);
    }
    if let Some(scroll) = scroll {
        intent.cycle += scroll_steps(scroll.delta.y, scroll.unit, &mut scroll_carry);
    }
    intent.aim = match (windows.single(), rigs.single()) {
        (Ok(window), Ok(rig)) => window
            .cursor_position()
            .map(|cursor| cursor_to_world(cursor, window.size(), rig.anchor)),
        _ => None,
    };
}

/// New facing toward `aim`, or `current` when the aim is on top of the player.
pub fn facing_toward(pos: Vec2, aim: Vec2, current: Vec2) -> Vec2 {
    let to = aim - pos;
    if to.length_squared() > 16.0 {
        to.normalize()
    } else {
        current
    }
}

fn move_player(
    time: Res<Time>,
    intent: Res<PlayerIntent>,
    colliders: Res<Colliders>,
    upgrades: Res<Upgrades>,
    doors: Query<&Door>,
    mut players: Query<(&mut Transform, &mut Movement, Has<Locked>), With<Player>>,
) {
    let dt = time.delta_secs();
    let dir = intent.direction.normalize_or_zero();
    for (mut transform, mut movement, locked) in &mut players {
        let walking = !locked && dir != Vec2::ZERO;
        if movement.walking != walking {
            movement.walking = walking;
        }
        if !walking {
            continue;
        }
        movement.heading = dir;
        let pos = transform.translation.truncate();
        let mut delta = dir * PLAYER_SPEED * upgrades.speed_factor() * dt;
        let mut solid = colliders.0.clone();
        for door in &doors {
            delta += door_assist(pos, dir, door, dt);
            solid.extend(door.leaf());
        }
        let next = move_circle(pos, delta, PLAYER_RADIUS, &solid);
        transform.translation = next.extend(transform.translation.z);
    }
}

fn face_aim(
    intent: Res<PlayerIntent>,
    mut players: Query<(&Transform, &mut Facing), With<Player>>,
) {
    let Some(aim) = intent.aim else {
        return;
    };
    for (transform, mut facing) in &mut players {
        facing.0 = facing_toward(transform.translation.truncate(), aim, facing.0);
    }
}

/// The closest interactable whose range covers `pos`.
pub fn nearest_interactable(
    pos: Vec2,
    items: impl IntoIterator<Item = (Vec2, Interactable)>,
) -> Option<InteractKind> {
    items
        .into_iter()
        .map(|(p, i)| (p.distance(pos), i))
        .filter(|(d, i)| *d <= i.range)
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, i)| i.kind)
}

fn update_focus(
    players: Query<&Transform, With<Player>>,
    items: Query<(&Transform, &Interactable)>,
    mut focus: ResMut<Focus>,
) {
    let target = players.iter().next().and_then(|player| {
        nearest_interactable(
            player.translation.truncate(),
            items.iter().map(|(t, i)| (t.translation.truncate(), *i)),
        )
    });
    if focus.0 != target {
        focus.0 = target;
    }
}

fn send_interact(
    mut intent: ResMut<PlayerIntent>,
    focus: Res<Focus>,
    mut out: MessageWriter<InteractPressed>,
) {
    if intent.interact_presses > 0 {
        // Several presses inside one tick still toggle only once.
        intent.interact_presses = 0;
        out.write(InteractPressed { target: focus.0 });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opposite_keys_cancel() {
        let mut keys = ButtonInput::<KeyCode>::default();
        keys.press(KeyCode::KeyA);
        keys.press(KeyCode::KeyD);
        assert_eq!(direction_from_keys(&keys), Vec2::ZERO);
    }

    #[test]
    fn wasd_maps_to_axes() {
        let mut keys = ButtonInput::<KeyCode>::default();
        keys.press(KeyCode::KeyW);
        keys.press(KeyCode::KeyD);
        assert_eq!(direction_from_keys(&keys), Vec2::new(1.0, 1.0));
    }

    #[test]
    fn number_keys_pick_slots() {
        let mut keys = ButtonInput::<KeyCode>::default();
        keys.press(KeyCode::Digit2);
        assert_eq!(slot_from_keys(&keys), Some(1));
        let mut keys = ButtonInput::<KeyCode>::default();
        keys.press(KeyCode::Numpad1);
        assert_eq!(slot_from_keys(&keys), Some(0));
    }

    #[test]
    fn one_wheel_notch_is_one_step() {
        let mut carry = 0.0;
        assert_eq!(scroll_steps(-1.0, MouseScrollUnit::Line, &mut carry), 1);
        assert_eq!(scroll_steps(3.0, MouseScrollUnit::Line, &mut carry), -1);
        assert_eq!(scroll_steps(-100.0, MouseScrollUnit::Pixel, &mut carry), 1);
    }

    #[test]
    fn pixel_scroll_accumulates_until_a_step() {
        let mut carry = 0.0;
        assert_eq!(scroll_steps(-40.0, MouseScrollUnit::Pixel, &mut carry), 0);
        assert_eq!(scroll_steps(-40.0, MouseScrollUnit::Pixel, &mut carry), 1);
        // A frame without scrolling drops the remainder.
        assert_eq!(scroll_steps(0.0, MouseScrollUnit::Pixel, &mut carry), 0);
        assert_eq!(carry, 0.0);
    }

    #[test]
    fn facing_follows_the_aim_but_not_onto_the_player() {
        assert_eq!(
            facing_toward(Vec2::ZERO, Vec2::new(0.0, 50.0), Vec2::X),
            Vec2::Y
        );
        assert_eq!(
            facing_toward(Vec2::ZERO, Vec2::new(1.0, 1.0), Vec2::X),
            Vec2::X
        );
    }

    #[test]
    fn nearest_interactable_in_range_wins() {
        let helm = Interactable {
            kind: InteractKind::Helm,
            range: 50.0,
        };
        let diag = Interactable {
            kind: InteractKind::Diagnostics,
            range: 100.0,
        };
        let items = [(Vec2::new(40.0, 0.0), helm), (Vec2::new(30.0, 0.0), diag)];
        assert_eq!(
            nearest_interactable(Vec2::ZERO, items),
            Some(InteractKind::Diagnostics)
        );
        assert_eq!(nearest_interactable(Vec2::new(500.0, 0.0), items), None);
    }
}
