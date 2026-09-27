//! Hand tools, always in the engineer's hand (drawn by the engineer's
//! sprite): the wrench and the tape roll. Number keys or the scroll wheel
//! pick one; left click uses it, but only standing still. Walking stops any
//! use at once (a turn is cancelled, tape stops).
//!
//! - Wrench: its bite (where the sprite's jaw is, per facing) snaps (a soft
//!   magnet) to any [`WrenchTarget`] within `SNAP_RADIUS`. A click on a
//!   snapped target turns it for `WRENCH_TURN_SECS`, then sends
//!   [`WrenchTightened`]. Pulling off the target cancels the turn.
//! - Tape: while the button is held and the roll's edge touches a wall,
//!   tape comes off the roll (one second of tape per second) and
//!   [`TapeLaid`] is sent. Tape on bare wall is wasted; the roll is finite.

use bevy::prelude::*;

use crate::art::engineer::{contact, facing};
use crate::player::{Facing, Locked, Movement, Player, PlayerIntent};
use crate::shapes::{Shapes, at, rect};
use crate::ship::Walls;
use crate::ship::layout::{WallContact, wall_contact};
use crate::{AppState, GameSet, RunEntity, RunSet, not_paused, palette, running};

/// The bite snaps to a target within this distance of where it rests.
pub const SNAP_RADIUS: f32 = 30.0;
/// One click turns a bolt for this long.
pub const WRENCH_TURN_SECS: f32 = 0.9;
/// The roll lays tape on a wall within this distance of its edge.
pub const TAPE_CONTACT: f32 = 14.0;
/// Seconds of taping on a full roll.
pub const TAPE_CAPACITY: f32 = 20.0;
/// Seconds of tape per visible strip on the wall.
const TAPE_STRIP_EVERY: f32 = 0.15;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Wrench,
    Tape,
}

impl Tool {
    /// Belt order: key 1 is the wrench, key 2 the tape.
    pub const BELT: [Tool; 2] = [Tool::Wrench, Tool::Tape];

    pub fn as_str(self) -> &'static str {
        match self {
            Tool::Wrench => "Wrench",
            Tool::Tape => "Tape",
        }
    }

    pub fn slot(self) -> usize {
        Tool::BELT.iter().position(|&t| t == self).unwrap_or(0)
    }
}

/// The belt picked `cycle` steps from `held` (wrapping).
pub fn cycle_tool(held: Tool, cycle: i32) -> Tool {
    let n = Tool::BELT.len() as i32;
    Tool::BELT[(held.slot() as i32 + cycle).rem_euclid(n) as usize]
}

#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct ToolBelt {
    pub held: Tool,
    /// Seconds of tape left on the roll.
    pub tape_left: f32,
}

impl Default for ToolBelt {
    fn default() -> Self {
        Self {
            held: Tool::Wrench,
            tape_left: TAPE_CAPACITY,
        }
    }
}

/// Something the wrench head snaps to (a loose bolt), at `pos`.
#[derive(Component, Debug, Clone, Copy)]
pub struct WrenchTarget {
    pub pos: Vec2,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WrenchTurn {
    pub target: Entity,
    pub left: f32,
}

/// Where the held tool is this tick. Read by fault repairs and visuals.
#[derive(Resource, Debug, Default, Clone)]
pub struct ToolState {
    /// Where the tool's working end rests (the sprite's contact point).
    pub tip: Vec2,
    /// Wrench bite after snapping (equals `tip` when not snapped).
    pub head: Vec2,
    /// Target the wrench head is snapped to.
    pub snap: Option<Entity>,
    /// Wrench turn in progress.
    pub turn: Option<WrenchTurn>,
    /// Wall contact while tape is being laid this tick.
    pub taping: Option<WallContact>,
    /// Tape laid since the last strip was spawned.
    strip_carry: f32,
    /// Strips spawned this run (varies their angle deterministically).
    strips: u32,
}

/// A bolt finished its turn.
#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct WrenchTightened {
    pub target: Entity,
}

/// `secs` of tape went onto the wall at `point` this tick.
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct TapeLaid {
    pub point: Vec2,
    pub normal: Vec2,
    pub secs: f32,
}

/// A strip of tape stuck to a wall.
#[derive(Component, Debug)]
pub struct TapeStrip;

/// The ring round the bolt the wrench is snapped to.
#[derive(Component, Debug)]
struct SnapRing;

pub struct ToolsPlugin;

impl Plugin for ToolsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ToolBelt>()
            .init_resource::<ToolState>()
            .add_message::<WrenchTightened>()
            .add_message::<TapeLaid>()
            .add_systems(
                OnEnter(AppState::Playing),
                reset_tools.in_set(RunSet::Spawn),
            )
            .add_systems(
                FixedUpdate,
                (select_tool, aim_tool, use_wrench, use_tape)
                    .chain()
                    .in_set(GameSet::Act)
                    .run_if(running),
            )
            .add_systems(
                Update,
                draw_snap.in_set(GameSet::Present).run_if(not_paused),
            );
    }
}

fn reset_tools(
    mut commands: Commands,
    shapes: Res<Shapes>,
    mut belt: ResMut<ToolBelt>,
    mut state: ResMut<ToolState>,
) {
    *belt = ToolBelt::default();
    *state = ToolState::default();
    commands.spawn((
        SnapRing,
        RunEntity,
        shapes.ring(13.0, palette::SNAP),
        at(Vec2::ZERO, 3.2),
        Visibility::Hidden,
    ));
}

fn select_tool(
    mut intent: ResMut<PlayerIntent>,
    mut belt: ResMut<ToolBelt>,
    mut state: ResMut<ToolState>,
) {
    let mut held = belt.held;
    if let Some(slot) = intent.select.take()
        && let Some(&tool) = Tool::BELT.get(slot)
    {
        held = tool;
    }
    if intent.cycle != 0 {
        held = cycle_tool(held, intent.cycle);
        intent.cycle = 0;
    }
    if held != belt.held {
        belt.held = held;
        state.turn = None;
    }
}

/// The target nearest `tip` within [`SNAP_RADIUS`].
pub fn snap_target(
    tip: Vec2,
    targets: impl IntoIterator<Item = (Entity, Vec2)>,
) -> Option<(Entity, Vec2)> {
    targets
        .into_iter()
        .map(|(e, p)| (e, p, p.distance(tip)))
        .filter(|(_, _, d)| *d <= SNAP_RADIUS)
        .min_by(|a, b| a.2.total_cmp(&b.2))
        .map(|(e, p, _)| (e, p))
}

/// Where the held tool's working end is for an engineer at `pos` facing
/// `look` (16 directions standing, 8 walking, as the sprite draws it).
pub fn tool_tip(tool: Tool, pos: Vec2, look: Vec2, walking: bool) -> Vec2 {
    let dir16 = facing(look, if walking { 8 } else { 16 }).unwrap_or(0);
    pos + contact(tool == Tool::Wrench, dir16)
}

fn aim_tool(
    belt: Res<ToolBelt>,
    mut state: ResMut<ToolState>,
    players: Query<(&Transform, &Facing, &Movement, Has<Locked>), With<Player>>,
    targets: Query<(Entity, &WrenchTarget)>,
) {
    let Some((transform, facing, movement, locked)) = players.iter().next() else {
        return;
    };
    let look = if movement.walking {
        movement.heading
    } else {
        facing.0
    };
    let pos = transform.translation.truncate();
    state.tip = tool_tip(belt.held, pos, look, movement.walking);
    let snapped = (belt.held == Tool::Wrench && !locked && !movement.walking)
        .then(|| snap_target(state.tip, targets.iter().map(|(e, t)| (e, t.pos))))
        .flatten();
    state.snap = snapped.map(|(e, _)| e);
    state.head = snapped.map_or(state.tip, |(_, p)| p);
}

fn use_wrench(
    time: Res<Time>,
    belt: Res<ToolBelt>,
    mut intent: ResMut<PlayerIntent>,
    mut state: ResMut<ToolState>,
    players: Query<&Movement, With<Player>>,
    mut out: MessageWriter<WrenchTightened>,
) {
    // Clicks never queue up: each tick consumes whatever arrived.
    let clicked = std::mem::take(&mut intent.use_presses) > 0;
    if belt.held != Tool::Wrench {
        return;
    }
    // No tool use on the move: walking cancels a turn, and clicks are ignored.
    if players.iter().any(|m| m.walking) {
        state.turn = None;
        return;
    }
    let snap = state.snap;
    match &mut state.turn {
        Some(turn) if snap != Some(turn.target) => state.turn = None,
        Some(turn) => {
            turn.left -= time.delta_secs();
            if turn.left <= 1e-4 {
                out.write(WrenchTightened {
                    target: turn.target,
                });
                state.turn = None;
            }
        }
        None => {
            if clicked && let Some(target) = snap {
                state.turn = Some(WrenchTurn {
                    target,
                    left: WRENCH_TURN_SECS,
                });
            }
        }
    }
}

fn use_tape(
    mut commands: Commands,
    time: Res<Time>,
    intent: Res<PlayerIntent>,
    walls: Res<Walls>,
    mut belt: ResMut<ToolBelt>,
    mut state: ResMut<ToolState>,
    players: Query<&Movement, (With<Player>, Without<Locked>)>,
    mut out: MessageWriter<TapeLaid>,
) {
    state.taping = None;
    let standing = players.iter().next().is_some_and(|m| !m.walking);
    if belt.held != Tool::Tape || !intent.use_held || belt.tape_left <= 0.0 || !standing {
        return;
    }
    let Some(contact) = wall_contact(state.tip, TAPE_CONTACT, &walls.0) else {
        return;
    };
    let secs = time.delta_secs().min(belt.tape_left);
    belt.tape_left -= secs;
    state.taping = Some(contact);
    out.write(TapeLaid {
        point: contact.point,
        normal: contact.normal,
        secs,
    });
    state.strip_carry += secs;
    if state.strip_carry >= TAPE_STRIP_EVERY {
        state.strip_carry = 0.0;
        state.strips += 1;
        commands.spawn((TapeStrip, RunEntity, tape_strip(contact, state.strips)));
    }
}

/// A strip stuck on the wall face at `contact`, angled a little differently
/// each time.
fn tape_strip(contact: WallContact, n: u32) -> (Sprite, Transform) {
    // Deterministic jitter in [-0.5, 0.5): cosmetic, so no RNG draw.
    let jitter = (n.wrapping_mul(2_654_435_761) % 1000) as f32 / 1000.0 - 0.5;
    let tangent = contact.normal.perp();
    let pos = contact.point - contact.normal * 3.0 + tangent * jitter * 20.0;
    let angle = tangent.to_angle() + jitter * 0.5;
    (
        rect(Vec2::new(24.0, 8.0), palette::TAPE),
        Transform::from_translation(pos.extend(1.7)).with_rotation(Quat::from_rotation_z(angle)),
    )
}

/// The snapped bolt gets a ring, which fills in as the turn goes round.
fn draw_snap(
    time: Res<Time>,
    state: Res<ToolState>,
    mut rings: Query<(&mut Transform, &mut Sprite, &mut Visibility), With<SnapRing>>,
) {
    for (mut transform, mut sprite, mut visibility) in &mut rings {
        let show = state.snap.is_some();
        let want = if show {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *visibility != want {
            *visibility = want;
        }
        if !show {
            continue;
        }
        transform.translation = state.head.extend(transform.translation.z);
        let pulse = 0.75 + 0.25 * (time.elapsed_secs() * 6.0).sin();
        let turned = state.turn.map_or(0.0, |t| 1.0 - t.left / WRENCH_TURN_SECS);
        sprite.color = palette::SNAP.with_alpha(pulse * (1.0 - 0.6 * turned));
        sprite.custom_size = Some(Vec2::splat(26.0 - 8.0 * turned));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cycling_wraps_both_ways() {
        assert_eq!(cycle_tool(Tool::Wrench, 1), Tool::Tape);
        assert_eq!(cycle_tool(Tool::Tape, 1), Tool::Wrench);
        assert_eq!(cycle_tool(Tool::Wrench, -1), Tool::Tape);
        assert_eq!(cycle_tool(Tool::Wrench, 2), Tool::Wrench);
    }

    #[test]
    fn snap_picks_the_nearest_target_in_range() {
        let a = Entity::from_raw_u32(1).unwrap();
        let b = Entity::from_raw_u32(2).unwrap();
        let targets = [(a, Vec2::new(20.0, 0.0)), (b, Vec2::new(10.0, 0.0))];
        assert_eq!(
            snap_target(Vec2::ZERO, targets),
            Some((b, Vec2::new(10.0, 0.0)))
        );
        assert_eq!(snap_target(Vec2::new(0.0, 100.0), targets), None);
    }

    #[test]
    fn tape_strip_lies_along_the_wall() {
        let contact = WallContact {
            point: Vec2::new(0.0, 260.0),
            normal: Vec2::NEG_Y,
        };
        let (_, t) = tape_strip(contact, 3);
        assert!((t.translation.y - 263.0).abs() < 1e-3, "on the wall face");
    }
}
