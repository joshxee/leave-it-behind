//! Score: +1 per Space press. Crossing [`SCORE_THRESHOLD`] sends a message.

use bevy::prelude::*;

use crate::player::PlayerIntent;
use crate::{AppState, GameSet};

pub const SCORE_THRESHOLD: u32 = 10;

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Score(pub u32);

/// Sent every time the score changes.
#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScoreChanged {
    pub score: u32,
}

/// Sent once when the score first reaches [`SCORE_THRESHOLD`].
#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThresholdReached;

pub struct ScoringPlugin;

impl Plugin for ScoringPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Score>()
            .add_message::<ScoreChanged>()
            .add_message::<ThresholdReached>()
            .add_systems(
                FixedUpdate,
                apply_score_presses
                    .in_set(GameSet::Resolve)
                    .run_if(in_state(AppState::Playing)),
            );
    }
}

/// Pure scoring rule: returns the new score and whether the threshold was crossed.
pub fn add_points(score: u32, points: u32) -> (u32, bool) {
    let new = score.saturating_add(points);
    (new, score < SCORE_THRESHOLD && new >= SCORE_THRESHOLD)
}

fn apply_score_presses(
    mut intent: ResMut<PlayerIntent>,
    mut score: ResMut<Score>,
    mut changed: MessageWriter<ScoreChanged>,
    mut threshold: MessageWriter<ThresholdReached>,
) {
    if intent.score_presses == 0 {
        return;
    }
    let (new, crossed) = add_points(score.0, intent.score_presses);
    intent.score_presses = 0;
    score.0 = new;
    changed.write(ScoreChanged { score: new });
    if crossed {
        threshold.write(ThresholdReached);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn threshold_is_crossed_once() {
        assert_eq!(add_points(9, 1), (10, true));
        assert_eq!(add_points(10, 1), (11, false));
        assert_eq!(add_points(3, 1), (4, false));
    }

    #[test]
    fn system_consumes_presses() {
        let mut world = World::new();
        world.init_resource::<Score>();
        world.init_resource::<Messages<ScoreChanged>>();
        world.init_resource::<Messages<ThresholdReached>>();
        world.insert_resource(PlayerIntent {
            score_presses: 2,
            ..default()
        });

        world.run_system_once(apply_score_presses).unwrap();

        assert_eq!(world.resource::<Score>().0, 2);
        assert_eq!(world.resource::<PlayerIntent>().score_presses, 0);
    }
}
