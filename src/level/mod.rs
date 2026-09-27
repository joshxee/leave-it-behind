//! The flight: the fault schedule of the current level, the damage the
//! faults do, and the outcome (landing once every fault is fixed, or a
//! fault's clock running out). A finished flight is recorded in
//! [`Progress`]. The five levels are flown in order ([`campaign`]).

pub mod campaign;
pub mod def;
pub mod one;
pub mod progress;

use bevy::prelude::*;

pub use campaign::{LEVEL_COUNT, LEVEL_IDS, campaign, next_after, number_of};
pub use def::{Choice, Envelope, FaultSlot, LevelDef, MAX_FLIGHT_SECS, PlannedFault, TimeWindow};
pub use one::level_one;
pub use progress::{Damage, LastRun, LevelProgress, Progress, RunRecord};

use crate::faults::{Fault, FaultFailed, FaultFixed, Site, fault_bundle, resolve_faults};
use crate::{AppState, GameRng, GameSet, RunSet, running};

/// The level being flown. Level one until the campaign moves on (`menu`).
#[derive(Resource, Debug, Clone)]
pub struct CurrentLevel(pub LevelDef);

impl CurrentLevel {
    /// Where this level comes in the campaign (1-based; 0 for a level
    /// outside it, as tests may fly).
    pub fn number(&self) -> usize {
        number_of(&self.0.id).unwrap_or(0)
    }

    /// The level flown after this one, if any.
    pub fn next(&self) -> Option<LevelDef> {
        next_after(&self.0.id)
    }
}

/// Seconds from the last fault's fix to touchdown: the final approach, with
/// nothing left to break.
pub const TOUCHDOWN_SECS: f32 = 3.0;

/// Time since launch. The flight has no set length: it lands
/// [`TOUCHDOWN_SECS`] after the last of its faults is fixed.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct Journey {
    pub elapsed: f32,
    /// Until launch the fault schedule waits. A flight launches straight
    /// away unless level one's pre-flight check holds it (see `coach`).
    pub launched: bool,
    /// When the last fault was fixed (seconds after launch), once it is.
    pub cleared_at: Option<f32>,
}

impl Default for Journey {
    fn default() -> Self {
        Self {
            elapsed: 0.0,
            launched: true,
            cleared_at: None,
        }
    }
}

impl Journey {
    /// Every fault fixed: the ship is on its final approach.
    pub fn cleared(&self) -> bool {
        self.cleared_at.is_some()
    }

    pub fn arrived(&self) -> bool {
        self.cleared_at
            .is_some_and(|at| self.elapsed >= at + TOUCHDOWN_SECS)
    }
}

/// Faults rolled for this run that have not started yet, by start time. A
/// fault whose site is still busy waits until it is free.
#[derive(Resource, Debug, Default, Clone)]
pub struct FaultPlan {
    pub pending: Vec<PlannedFault>,
}

impl FaultPlan {
    /// Faults this flight must fix to land: those started and those to come.
    pub fn total(&self, stats: &RunStats) -> u32 {
        stats.started + self.pending.len() as u32
    }
}

#[derive(Resource, Debug, Default, Clone, PartialEq)]
pub struct RunStats {
    pub started: u32,
    pub fixed: u32,
    /// The fault that ended the run.
    pub failure: Option<Site>,
    /// How long each kind of fault ran before it was fixed.
    pub damage: Damage,
}

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Journey>()
            .insert_resource(CurrentLevel(level_one()))
            .init_resource::<FaultPlan>()
            .init_resource::<RunStats>()
            .init_resource::<Progress>()
            .init_resource::<LastRun>()
            .add_systems(OnEnter(AppState::Playing), start_run.in_set(RunSet::Spawn))
            .add_systems(
                FixedUpdate,
                (advance_journey, start_due_faults, track_damage)
                    .chain()
                    .in_set(GameSet::Simulate)
                    .run_if(running),
            )
            .add_systems(
                FixedUpdate,
                (count_fixes, decide_outcome)
                    .chain()
                    .after(resolve_faults)
                    .in_set(GameSet::Resolve)
                    .run_if(running),
            )
            .add_systems(OnEnter(AppState::Landed), record_run)
            .add_systems(OnEnter(AppState::Lost), record_run);
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
    *journey = Journey::default();
    *stats = RunStats::default();
}

fn advance_journey(time: Res<Time>, mut journey: ResMut<Journey>) {
    if journey.launched {
        journey.elapsed += time.delta_secs();
    }
}

fn start_due_faults(
    mut commands: Commands,
    journey: Res<Journey>,
    mut plan: ResMut<FaultPlan>,
    active: Query<&Fault>,
    mut stats: ResMut<RunStats>,
) {
    if !journey.launched {
        return;
    }
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

/// Every unfixed fault adds its tick to its kind's damage.
fn track_damage(time: Res<Time>, faults: Query<&Fault>, mut stats: ResMut<RunStats>) {
    for fault in &faults {
        if !fault.is_repaired() {
            stats.damage.add(fault.kind(), time.delta_secs());
        }
    }
}

fn count_fixes(mut fixed: MessageReader<FaultFixed>, mut stats: ResMut<RunStats>) {
    stats.fixed += fixed.read().count() as u32;
}

/// Lost the moment a fault fails. Once every planned fault has started and
/// been fixed the flight is cleared, and it lands after the final approach.
/// A flight whose plan never started a fault (the `quiet` scenario and those
/// built on it) never lands.
fn decide_outcome(
    mut failed: MessageReader<FaultFailed>,
    plan: Res<FaultPlan>,
    mut journey: ResMut<Journey>,
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
        return;
    }
    if journey.cleared_at.is_none()
        && plan.pending.is_empty()
        && stats.started > 0
        && stats.fixed >= stats.started
    {
        journey.cleared_at = Some(journey.elapsed);
    }
    if journey.arrived() {
        next.set(AppState::Landed);
    }
}

/// Scores the flight that just ended and keeps it if it is the level's best.
fn record_run(
    state: Res<State<AppState>>,
    level: Res<CurrentLevel>,
    journey: Res<Journey>,
    stats: Res<RunStats>,
    mut progress: ResMut<Progress>,
    mut last: ResMut<LastRun>,
) {
    let record = RunRecord {
        landed: *state.get() == AppState::Landed,
        damage: stats.damage,
        survived: journey.elapsed,
    };
    let new_best = progress.record(&level.0.id, record);
    *last = LastRun {
        record: Some(record),
        new_best,
    };
}

/// `m:ss`, rounding up.
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
    fn the_journey_lands_after_the_final_approach() {
        let mut j = Journey::default();
        assert!(j.launched, "flights launch unless held");
        j.elapsed = 500.0;
        assert!(!j.cleared() && !j.arrived(), "no set length");
        j.cleared_at = Some(40.0);
        j.elapsed = 40.0 + TOUCHDOWN_SECS - 0.1;
        assert!(j.cleared() && !j.arrived());
        j.elapsed = 40.0 + TOUCHDOWN_SECS;
        assert!(j.arrived());
    }

    #[test]
    fn the_plan_total_counts_started_and_pending() {
        let plan = FaultPlan {
            pending: level_one().roll(&mut GameRng::from_seed(0)),
        };
        let stats = RunStats {
            started: 2,
            ..default()
        };
        assert_eq!(plan.total(&stats), 6);
    }
}
