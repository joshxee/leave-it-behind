//! Hand tools, held visibly at the end of the engineer's arm: the wrench and
//! the tape roll. Number keys or the scroll wheel pick one; left click uses it.
//!
//! - Wrench: the head rests `WRENCH_REACH` ahead of the player and snaps (a
//!   soft magnet) to any [`WrenchTarget`] within `SNAP_RADIUS`. A click on a
//!   snapped target turns it for `WRENCH_TURN_SECS`, then sends
//!   [`WrenchTightened`]. Pulling the head off the target cancels the turn.
//! - Tape: while the button is held and the roll's tip touches a wall, tape
//!   comes off the roll (one second of tape per second) and [`TapeLaid`] is
//!   sent. Tape on bare wall is wasted; the roll is finite.

use bevy::prelude::*;

use crate::player::{Facing, Locked, Player, PlayerIntent};
use crate::shapes::{Shapes, at, rect};
use crate::ship::Walls;
use crate::ship::layout::{WallContact, wall_contact};
use crate::{AppState, GameSet, RunEntity, RunSet, palette};

/// Player center to the wrench head at rest.
pub const WRENCH_REACH: f32 = 46.0;
/// The head snaps to a target within this distance of its rest position.
pub const SNAP_RADIUS: f32 = 30.0;
/// One click turns a bolt for this long.
pub const WRENCH_TURN_SECS: f32 = 0.6;
/// Player center to the tape roll.
pub const TAPE_REACH: f32 = 36.0;
/// The roll lays tape on a wall within this distance of it.
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
    /// Rest position of the tool's working end.
    pub tip: Vec2,
    /// Wrench head position after snapping (equals `tip` when not snapped).
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

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
enum ToolPart {
    WrenchHandle,
    WrenchHead,
    TapeRoll,
    TapeRun,
}

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
                    .run_if(in_state(AppState::Playing)),
            )
            .add_systems(Update, draw_tools.in_set(GameSet::Present));
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
    let parts = [
        (
            ToolPart::WrenchHandle,
            rect(Vec2::new(30.0, 6.0), palette::STEEL),
            3.1,
        ),
        (ToolPart::WrenchHead, shapes.ring(10.0, palette::STEEL), 3.2),
        (ToolPart::TapeRoll, shapes.ring(12.0, palette::TAPE), 3.2),
        (
            ToolPart::TapeRun,
            rect(Vec2::new(10.0, 5.0), palette::TAPE),
            3.15,
        ),
    ];
    for (part, sprite, z) in parts {
        commands.spawn((
            part,
            RunEntity,
            sprite,
            at(Vec2::ZERO, z),
            Visibility::Hidden,
        ));
    }
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

fn aim_tool(
    belt: Res<ToolBelt>,
    mut state: ResMut<ToolState>,
    players: Query<(&Transform, &Facing, Has<Locked>), With<Player>>,
    targets: Query<(Entity, &WrenchTarget)>,
) {
    let Some((transform, facing, locked)) = players.iter().next() else {
        return;
    };
    let reach = match belt.held {
        Tool::Wrench => WRENCH_REACH,
        Tool::Tape => TAPE_REACH,
    };
    state.tip = transform.translation.truncate() + facing.0 * reach;
    let snapped = (belt.held == Tool::Wrench && !locked)
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
    mut out: MessageWriter<WrenchTightened>,
) {
    // Clicks never queue up: each tick consumes whatever arrived.
    let clicked = std::mem::take(&mut intent.use_presses) > 0;
    if belt.held != Tool::Wrench {
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
    players: Query<(), (With<Player>, Without<Locked>)>,
    mut out: MessageWriter<TapeLaid>,
) {
    state.taping = None;
    if belt.held != Tool::Tape || !intent.use_held || belt.tape_left <= 0.0 || players.is_empty() {
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

/// A strip along the wall at `contact`, angled a little differently each time.
fn tape_strip(contact: WallContact, n: u32) -> (Sprite, Transform) {
    // Deterministic jitter in [-0.5, 0.5): cosmetic, so no RNG draw.
    let jitter = (n.wrapping_mul(2_654_435_761) % 1000) as f32 / 1000.0 - 0.5;
    let tangent = contact.normal.perp();
    let pos = contact.point + contact.normal * 2.0 + tangent * jitter * 20.0;
    let angle = tangent.to_angle() + jitter * 0.9;
    (
        rect(Vec2::new(24.0, 8.0), palette::TAPE),
        Transform::from_translation(pos.extend(1.7)).with_rotation(Quat::from_rotation_z(angle)),
    )
}

/// How fast the drawn wrench head eases onto a snapped bolt (per second).
const MAGNET_EASE: f32 = 25.0;

fn draw_tools(
    time: Res<Time>,
    belt: Res<ToolBelt>,
    state: Res<ToolState>,
    players: Query<(&Transform, Has<Locked>), With<Player>>,
    mut parts: Query<(&ToolPart, &mut Transform, &mut Sprite, &mut Visibility), Without<Player>>,
    mut pull: Local<Vec2>,
) {
    let Some((player, locked)) = players.iter().next() else {
        return;
    };
    let pos = player.translation.truncate();
    // Soft magnet: the drawn head slides toward the snapped bolt instead of
    // jumping (the snap itself, and what a click turns, is exact).
    let target = state.head - state.tip;
    let eased = *pull + (target - *pull) * (1.0 - (-MAGNET_EASE * time.delta_secs()).exp());
    *pull = eased;
    let head = state.tip + *pull;
    for (part, mut transform, mut sprite, mut visibility) in &mut parts {
        let (held, from, to) = match part {
            ToolPart::WrenchHandle => (belt.held == Tool::Wrench, pos, head),
            ToolPart::WrenchHead => (belt.held == Tool::Wrench, head, head),
            ToolPart::TapeRoll => (belt.held == Tool::Tape, state.tip, state.tip),
            ToolPart::TapeRun => {
                let run = state.taping.map(|c| c.point);
                (run.is_some(), state.tip, run.unwrap_or(state.tip))
            }
        };
        // Hands are on the joystick while at the helm.
        let show = held && !locked;
        *visibility = if show {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if !show {
            continue;
        }
        let z = transform.translation.z;
        match part {
            ToolPart::WrenchHandle | ToolPart::TapeRun => {
                // A bar from `from` to `to`, starting at the player's edge for the handle.
                let dir = (to - from).normalize_or(Vec2::X);
                let start = if *part == ToolPart::WrenchHandle {
                    from + dir * 12.0
                } else {
                    from
                };
                let len = (to - start).length().max(2.0);
                transform.translation = ((start + to) / 2.0).extend(z);
                transform.rotation = Quat::from_rotation_z(dir.to_angle());
                let thickness = if *part == ToolPart::WrenchHandle {
                    6.0
                } else {
                    5.0
                };
                sprite.custom_size = Some(Vec2::new(len, thickness));
            }
            ToolPart::WrenchHead => {
                transform.translation = to.extend(z);
                sprite.color = match (state.turn, state.snap) {
                    (Some(turn), _) => {
                        palette::SNAP.mix(&palette::STEEL, turn.left / WRENCH_TURN_SECS)
                    }
                    (None, Some(_)) => palette::SNAP,
                    (None, None) => palette::STEEL,
                };
            }
            ToolPart::TapeRoll => {
                transform.translation = to.extend(z);
                let radius = 6.0 + 7.0 * (belt.tape_left / TAPE_CAPACITY).clamp(0.0, 1.0);
                sprite.custom_size = Some(Vec2::splat(radius * 2.0));
                sprite.color = if belt.tape_left > 0.0 {
                    palette::TAPE
                } else {
                    palette::UI_DIM
                };
            }
        }
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
        assert!((t.translation.y - 258.0).abs() < 1e-3);
    }
}
