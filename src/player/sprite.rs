//! The engineer's sprite (`assets/characters/engineer`), on a child entity
//! of the player so it can lean onto a bolt without moving the player.
//!
//! One animation at a time, cut cleanly (no blending, each starts at its
//! first frame):
//! - walking: `*_walk` with the held tool, facing the way they walk (8 directions);
//! - standing: `*_hold`, facing the mouse (16 directions), or the bolt the
//!   wrench is snapped to;
//! - using: `wrench_use` over the bolt's turn, `tape_use` looping while tape
//!   goes on, facing latched when the use starts (8 directions);
//! - at the helm: empty-handed `idle`, facing the window.

use bevy::prelude::*;
use bevy::sprite::Anchor;

use super::{Facing, Locked, Movement, Player};
use crate::GameSet;
use crate::art::Art;
use crate::art::engineer::{ANCHOR, Action, contact, facing};
use crate::faults::drift::Nav;
use crate::ship::layout;
use crate::tools::{Tool, ToolBelt, ToolState, WRENCH_TURN_SECS};

/// Most the sprite leans toward a snapped bolt, in world units.
pub const LEAN_MAX: f32 = 12.0;
/// How fast the lean eases in and out (per second).
const LEAN_EASE: f32 = 25.0;

/// The engineer's drawn pose: the current animation, the 16-direction
/// facing it is drawn at, and time since it started.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct EngineerSprite {
    pub action: Action,
    pub dir16: usize,
    pub clock: f32,
    pub lean: Vec2,
}

/// What decides the pose.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PoseInputs {
    pub held: Tool,
    pub at_helm: bool,
    /// Walking, and which way.
    pub walking: Option<Vec2>,
    /// Toward the mouse (unit vector).
    pub aim: Vec2,
    /// From the engineer to the bolt the wrench is snapped to.
    pub snapped: Option<Vec2>,
    /// Wrench turn progress, 0 to 1, while turning.
    pub turn: Option<f32>,
    pub taping: bool,
    /// From the helm seat toward the window.
    pub helm_facing: Vec2,
}

/// The animation to show and the direction to face (16-direction index;
/// eight-direction actions use even ones).
pub fn pose(i: &PoseInputs) -> (Action, Vec2, usize) {
    let (action, look, dirs) = if i.at_helm {
        (Action::Idle, i.helm_facing, 16)
    } else if let Some(heading) = i.walking {
        let walk = match i.held {
            Tool::Wrench => Action::WrenchWalk,
            Tool::Tape => Action::TapeWalk,
        };
        (walk, heading, 8)
    } else if i.turn.is_some() {
        (Action::WrenchUse, i.snapped.unwrap_or(i.aim), 8)
    } else if i.taping {
        (Action::TapeUse, i.aim, 8)
    } else {
        match (i.held, i.snapped) {
            (Tool::Wrench, Some(bolt)) => (Action::WrenchHold, bolt, 16),
            (Tool::Wrench, None) => (Action::WrenchHold, i.aim, 16),
            (Tool::Tape, _) => (Action::TapeHold, i.aim, 16),
        }
    };
    let dir16 = facing(look, dirs).unwrap_or(0);
    (action, look, dir16)
}

/// Frame of `action` after `clock` seconds, or from the wrench turn.
pub fn frame(action: Action, clock: f32, turn: Option<f32>) -> usize {
    match (action, turn) {
        (Action::WrenchUse, Some(progress)) => ((progress * 3.0) as usize).min(2),
        _ => (clock / action.frame_secs()) as usize % action.frames(),
    }
}

pub struct EngineerSpritePlugin;

impl Plugin for EngineerSpritePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, animate.in_set(GameSet::Present));
    }
}

/// The sprite child a new player gets: holding the wrench, facing down.
pub fn engineer_sprite(art: &Art) -> impl Bundle {
    (
        EngineerSprite {
            action: Action::WrenchHold,
            dir16: 4,
            clock: 0.0,
            lean: Vec2::ZERO,
        },
        art.engineer(Action::WrenchHold.atlas_index(4, 0)),
        Anchor(ANCHOR),
        Transform::default(),
    )
}

fn animate(
    time: Res<Time>,
    belt: Res<ToolBelt>,
    tools: Res<ToolState>,
    nav: Res<Nav>,
    players: Query<(&Transform, &Facing, &Movement, Has<Locked>), With<Player>>,
    mut sprites: Query<
        (&ChildOf, &mut EngineerSprite, &mut Sprite, &mut Transform),
        Without<Player>,
    >,
) {
    let dt = time.delta_secs();
    for (parent, mut look, mut sprite, mut transform) in &mut sprites {
        let Ok((player, aim, movement, locked)) = players.get(parent.parent()) else {
            continue;
        };
        let root = player.translation.truncate();
        let snapped = tools.snap.map(|_| tools.head - root);
        let inputs = PoseInputs {
            held: belt.held,
            at_helm: locked || nav.engaged,
            walking: movement.walking.then_some(movement.heading),
            aim: aim.0,
            snapped,
            turn: tools.turn.map(|t| 1.0 - t.left / WRENCH_TURN_SECS),
            taping: tools.taping.is_some(),
            helm_facing: layout::ship().helm().facing,
        };
        let (action, _, dir16) = pose(&inputs);
        if action != look.action {
            // A clean cut: the new animation starts at its first frame.
            look.action = action;
            look.clock = 0.0;
            look.dir16 = dir16;
        } else {
            look.clock += dt;
            // Uses keep the facing they started with.
            if !matches!(action, Action::WrenchUse | Action::TapeUse) {
                look.dir16 = dir16;
            }
        }
        let index = action.atlas_index(look.dir16, frame(action, look.clock, inputs.turn));
        if let Some(atlas) = sprite.texture_atlas.as_mut()
            && atlas.index != index
        {
            atlas.index = index;
        }
        // Lean so the wrench's bite lands on the snapped bolt.
        let target = match (snapped, action) {
            (Some(bolt), Action::WrenchHold | Action::WrenchUse) => {
                (bolt - contact(true, look.dir16)).clamp_length_max(LEAN_MAX)
            }
            _ => Vec2::ZERO,
        };
        let lean = look.lean;
        look.lean = lean + (target - lean) * (1.0 - (-LEAN_EASE * dt).exp());
        transform.translation.x = look.lean.x;
        transform.translation.y = look.lean.y;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn standing() -> PoseInputs {
        PoseInputs {
            held: Tool::Wrench,
            at_helm: false,
            walking: None,
            aim: Vec2::X,
            snapped: None,
            turn: None,
            taping: false,
            helm_facing: Vec2::Y,
        }
    }

    #[test]
    fn walking_carries_the_tool_and_faces_the_way_walked() {
        let walking = PoseInputs {
            walking: Some(Vec2::NEG_Y),
            ..standing()
        };
        assert_eq!(pose(&walking).0, Action::WrenchWalk);
        assert_eq!(pose(&walking).2, 4, "south, not toward the mouse");
        let tape = PoseInputs {
            held: Tool::Tape,
            ..walking
        };
        assert_eq!(pose(&tape).0, Action::TapeWalk);
    }

    #[test]
    fn walking_wins_over_using() {
        // Tools stop the moment the engineer walks: a turn or taping in
        // progress never shows while walking.
        let i = PoseInputs {
            walking: Some(Vec2::X),
            turn: Some(0.5),
            taping: true,
            ..standing()
        };
        assert_eq!(pose(&i).0, Action::WrenchWalk);
    }

    #[test]
    fn standing_faces_the_mouse_or_the_snapped_bolt() {
        assert_eq!(pose(&standing()), (Action::WrenchHold, Vec2::X, 0));
        let snapped = PoseInputs {
            snapped: Some(Vec2::new(0.0, 30.0)),
            ..standing()
        };
        assert_eq!(pose(&snapped).2, 12, "north, toward the bolt");
    }

    #[test]
    fn uses_and_the_helm_have_their_own_poses() {
        let turning = PoseInputs {
            turn: Some(0.2),
            snapped: Some(Vec2::new(-20.0, 0.0)),
            ..standing()
        };
        assert_eq!(pose(&turning).0, Action::WrenchUse);
        assert_eq!(pose(&turning).2, 8, "west, toward the bolt");
        let taping = PoseInputs {
            held: Tool::Tape,
            taping: true,
            ..standing()
        };
        assert_eq!(pose(&taping).0, Action::TapeUse);
        let helm = PoseInputs {
            at_helm: true,
            ..taping
        };
        assert_eq!(pose(&helm), (Action::Idle, Vec2::Y, 12));
    }

    #[test]
    fn frames_follow_the_clock_or_the_turn() {
        assert_eq!(frame(Action::WrenchWalk, 0.0, None), 0);
        assert_eq!(frame(Action::WrenchWalk, 0.12, None), 1);
        assert_eq!(frame(Action::WrenchWalk, 0.45, None), 0, "the walk loops");
        assert_eq!(frame(Action::WrenchUse, 0.0, Some(0.1)), 0);
        assert_eq!(frame(Action::WrenchUse, 0.0, Some(0.5)), 1);
        assert_eq!(frame(Action::WrenchUse, 0.0, Some(1.0)), 2);
        assert_eq!(frame(Action::TapeUse, 0.2, None), 2);
        assert_eq!(frame(Action::TapeHold, 5.0, None), 0);
    }
}
