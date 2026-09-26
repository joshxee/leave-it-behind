//! The flight: the countdown to arrival, the fault schedule of the current
//! level, and the outcome (landing, or a fault's clock running out). R
//! restarts from the landed or lost screen.

pub mod def;
pub mod one;

use bevy::prelude::*;

pub use def::{Choice, Envelope, FaultSlot, LevelDef, PlannedFault, TimeWindow};
pub use one::level_one;

use crate::faults::{Fault, FaultFailed, FaultFixed, Site, fault_bundle, resolve_faults};
use crate::{AppState, GameRng, GameSet, RunSet};

/// The level being flown.
#[derive(Resource, Debug, Clone)]
pub struct CurrentLevel(pub LevelDef);

/// Time since launch. The HUD counts `remaining` down to the landing.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct Journey {
    pub duration: f32,
    pub elapsed: f32,
}

impl Journey {
    pub fn new(duration: f32) -> Self {
        Self {
            duration,
            elapsed: 0.0,
        }
    }

    pub fn remaining(&self) -> f32 {
        (self.duration - self.elapsed).max(0.0)
    }

    pub fn arrived(&self) -> bool {
        self.elapsed >= self.duration
    }
}

/// Faults rolled for this run that have not started yet, by start time. A
/// fault whose site is still busy waits until it is free.
#[derive(Resource, Debug, Default, Clone)]
pub struct FaultPlan {
    pub pending: Vec<PlannedFault>,
}

#[derive(Resource, Debug, Default, Clone, PartialEq)]
pub struct RunStats {
    pub started: u32,
    pub fixed: u32,
    /// The fault that ended the run.
    pub failure: Option<Site>,
}

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        let level = level_one();
        app.insert_resource(Journey::new(level.duration_secs))
            .insert_resource(CurrentLevel(level))
            .init_resource::<FaultPlan>()
            .init_resource::<RunStats>()
            .add_systems(OnEnter(AppState::Playing), start_run.in_set(RunSet::Spawn))
            .add_systems(
                FixedUpdate,
                (advance_journey, start_due_faults)
                    .chain()
                    .in_set(GameSet::Simulate)
                    .run_if(in_state(AppState::Playing)),
            )
            .add_systems(
                FixedUpdate,
                (count_fixes, decide_outcome)
                    .chain()
                    .after(resolve_faults)
                    .in_set(GameSet::Resolve)
                    .run_if(in_state(AppState::Playing)),
            )
            .add_systems(
                Update,
                restart
                    .in_set(GameSet::Input)
                    .run_if(in_state(AppState::Landed).or_else(in_state(AppState::Lost))),
            );
    }
}

fn start_run(
    level: Res<CurrentLevel>,
    mut rng: ResMut<GameRng>,
    mut journey: ResMut<Journey>,
    mut plan: ResMut<FaultPlan>,
    mut stats: ResMut<RunStats>,
) {
    if let Some(seed) = level.0.seed {
        *rng = GameRng::from_seed(seed);
    }
    plan.pending = level.0.roll(&mut rng);
    *journey = Journey::new(level.0.duration_secs);
    *stats = RunStats::default();
}

fn advance_journey(time: Res<Time>, mut journey: ResMut<Journey>) {
    journey.elapsed += time.delta_secs();
}

fn start_due_faults(
    mut commands: Commands,
    journey: Res<Journey>,
    mut plan: ResMut<FaultPlan>,
    active: Query<&Fault>,
    mut stats: ResMut<RunStats>,
) {
    let mut busy: Vec<Site> = active.iter().map(|f| f.site).collect();
    let mut i = 0;
    while i < plan.pending.len() {
        let due = plan.pending[i];
        if due.at > journey.elapsed {
            break;
        }
        if busy.contains(&due.site) {
            i += 1;
            continue;
        }
        commands.spawn(fault_bundle(due.site, due.clock));
        busy.push(due.site);
        stats.started += 1;
        plan.pending.remove(i);
    }
}

fn count_fixes(mut fixed: MessageReader<FaultFixed>, mut stats: ResMut<RunStats>) {
    stats.fixed += fixed.read().count() as u32;
}

fn decide_outcome(
    mut failed: MessageReader<FaultFailed>,
    journey: Res<Journey>,
    mut stats: ResMut<RunStats>,
    mut next: ResMut<NextState<AppState>>,
) {
    let mut failure = None;
    for f in failed.read() {
        failure.get_or_insert(f.site);
    }
    if let Some(site) = failure {
        stats.failure = Some(site);
        next.set(AppState::Lost);
    } else if journey.arrived() {
        next.set(AppState::Landed);
    }
}

fn restart(keys: Res<ButtonInput<KeyCode>>, mut next: ResMut<NextState<AppState>>) {
    if keys.just_pressed(KeyCode::KeyR) {
        next.set(AppState::Playing);
    }
}

/// `m:ss`, rounding up so the clock reads 0:00 only on arrival.
pub fn format_clock(secs: f32) -> String {
    let total = secs.max(0.0).ceil() as u32;
    format!("{}:{:02}", total / 60, total % 60)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clock_rounds_up() {
        assert_eq!(format_clock(240.0), "4:00");
        assert_eq!(format_clock(239.2), "4:00");
        assert_eq!(format_clock(59.01), "1:00");
        assert_eq!(format_clock(0.2), "0:01");
        assert_eq!(format_clock(0.0), "0:00");
        assert_eq!(format_clock(-3.0), "0:00");
    }

    #[test]
    fn journey_counts_down() {
        let mut j = Journey::new(10.0);
        j.elapsed = 4.0;
        assert_eq!(j.remaining(), 6.0);
        assert!(!j.arrived());
        j.elapsed = 10.0;
        assert!(j.arrived());
        assert_eq!(j.remaining(), 0.0);
    }
}
