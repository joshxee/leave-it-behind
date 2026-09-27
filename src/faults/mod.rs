//! Faults: what breaks, where, and how long until it becomes fatal.
//!
//! A fault is an entity with a [`Fault`] component: its [`Site`] (which
//! fixes its kind and room), a failure clock, and repair progress. Spawning
//! one is all it takes to start it; each kind's module reacts to
//! `Added<Fault>` (loosening bolts, opening the breach, starting the drift).
//! [`resolve_faults`] despawns repaired faults ([`FaultFixed`]) and reports
//! expired ones ([`FaultFailed`]). The three kinds have independent clocks.

pub mod bolts;
pub mod breach;
pub mod drift;
pub mod sites;

use bevy::prelude::*;

pub use sites::Site;

use crate::{AppState, GameSet, RunEntity};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FaultKind {
    /// Engine room, fixed with the wrench. Unfixed, the engine overheats.
    LooseBolts,
    /// Airlock and main hull walls, fixed with tape. Unfixed, the oxygen runs out.
    HullBreach,
    /// The nav helm, fixed by steering. Unfixed, the ship hits an asteroid.
    TrajectoryDrift,
}

impl FaultKind {
    pub const ALL: [FaultKind; 3] = [
        FaultKind::LooseBolts,
        FaultKind::HullBreach,
        FaultKind::TrajectoryDrift,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            FaultKind::LooseBolts => "LooseBolts",
            FaultKind::HullBreach => "HullBreach",
            FaultKind::TrajectoryDrift => "TrajectoryDrift",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            FaultKind::LooseBolts => "Loose bolts",
            FaultKind::HullBreach => "Hull breach",
            FaultKind::TrajectoryDrift => "Trajectory drift",
        }
    }

    /// Headline when this fault's clock runs out.
    pub fn failure_title(self) -> &'static str {
        match self {
            FaultKind::LooseBolts => "Engine overheated",
            FaultKind::HullBreach => "Oxygen depleted",
            FaultKind::TrajectoryDrift => "Asteroid collision",
        }
    }

    pub fn failure_text(self) -> &'static str {
        match self {
            FaultKind::LooseBolts => "The loose bolts shook the engine until it cooked itself.",
            FaultKind::HullBreach => "The breach vented the last of the air into space.",
            FaultKind::TrajectoryDrift => "The ship drifted off course, straight into a rock.",
        }
    }

    /// Failure clock range in seconds when a level does not pin it.
    pub fn clock_range(self) -> (f32, f32) {
        match self {
            FaultKind::LooseBolts => (60.0, 70.0),
            FaultKind::HullBreach => (55.0, 65.0),
            FaultKind::TrajectoryDrift => (60.0, 75.0),
        }
    }

    pub fn sites(self) -> &'static [Site] {
        match self {
            FaultKind::LooseBolts => &sites::BOLT_SITES,
            FaultKind::HullBreach => &sites::BREACH_SITES,
            FaultKind::TrajectoryDrift => &sites::DRIFT_SITES,
        }
    }
}

/// Shortest and longest failure clock any fault may have, in seconds.
pub const CLOCK_LIMITS: (f32, f32) = (55.0, 90.0);

/// An active fault.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
#[require(RunEntity, Transform, Visibility)]
pub struct Fault {
    pub site: Site,
    /// Seconds from the fault starting to it becoming fatal.
    pub clock: f32,
    /// Seconds since the fault started.
    pub elapsed: f32,
    /// Repair progress, 0 to 1. The fault is fixed at 1.
    pub repair: f32,
}

impl Fault {
    pub fn new(site: Site, clock: f32) -> Self {
        Self {
            site,
            clock,
            elapsed: 0.0,
            repair: 0.0,
        }
    }

    pub fn kind(&self) -> FaultKind {
        self.site.kind()
    }

    pub fn remaining(&self) -> f32 {
        (self.clock - self.elapsed).max(0.0)
    }

    /// How close to failing, 0 (just started) to 1 (fatal).
    pub fn urgency(&self) -> f32 {
        (self.elapsed / self.clock).clamp(0.0, 1.0)
    }

    pub fn is_repaired(&self) -> bool {
        self.repair >= 1.0
    }

    pub fn has_failed(&self) -> bool {
        !self.is_repaired() && self.elapsed >= self.clock
    }
}

/// Everything needed to start a fault now. Spawn it.
pub fn fault_bundle(site: Site, clock: f32) -> impl Bundle {
    (
        Fault::new(site, clock),
        Name::new(format!("{} at {}", site.kind().name(), site.as_str())),
        Transform::from_translation(site.pos().extend(0.0)),
    )
}

#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct FaultFixed {
    pub site: Site,
    /// Seconds from the fault starting to its repair.
    pub took: f32,
}

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct FaultFailed {
    pub site: Site,
}

pub struct FaultsPlugin;

impl Plugin for FaultsPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<FaultFixed>()
            .add_message::<FaultFailed>()
            .add_plugins((bolts::BoltsPlugin, breach::BreachPlugin, drift::DriftPlugin))
            .add_systems(
                FixedUpdate,
                tick_fault_clocks
                    .in_set(GameSet::Simulate)
                    .run_if(in_state(AppState::Playing)),
            )
            .add_systems(
                FixedUpdate,
                resolve_faults
                    .in_set(GameSet::Resolve)
                    .run_if(in_state(AppState::Playing)),
            );
    }
}

fn tick_fault_clocks(time: Res<Time>, mut faults: Query<&mut Fault>) {
    for mut fault in &mut faults {
        fault.elapsed += time.delta_secs();
    }
}

/// Repaired faults are despawned and reported; expired ones are reported
/// (the level turns that into a loss). A repair on the last tick counts.
pub fn resolve_faults(
    mut commands: Commands,
    faults: Query<(Entity, &Fault)>,
    mut fixed: MessageWriter<FaultFixed>,
    mut failed: MessageWriter<FaultFailed>,
) {
    for (entity, fault) in &faults {
        if fault.is_repaired() {
            fixed.write(FaultFixed {
                site: fault.site,
                took: fault.elapsed,
            });
            commands.entity(entity).despawn();
        } else if fault.has_failed() {
            failed.write(FaultFailed { site: fault.site });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_clocks_stay_within_limits() {
        for kind in FaultKind::ALL {
            let (lo, hi) = kind.clock_range();
            assert!(
                CLOCK_LIMITS.0 <= lo && lo <= hi && hi <= CLOCK_LIMITS.1,
                "{kind:?}"
            );
        }
    }

    #[test]
    fn repair_on_the_last_tick_is_not_a_failure() {
        let mut f = Fault::new(Site::Helm, 50.0);
        f.elapsed = 50.0;
        assert!(f.has_failed());
        f.repair = 1.0;
        assert!(!f.has_failed());
    }

    #[test]
    fn urgency_runs_from_zero_to_one() {
        let mut f = Fault::new(Site::HullPortAft, 40.0);
        assert_eq!(f.urgency(), 0.0);
        f.elapsed = 10.0;
        assert_eq!(f.urgency(), 0.25);
        assert_eq!(f.remaining(), 30.0);
        f.elapsed = 99.0;
        assert_eq!(f.urgency(), 1.0);
        assert_eq!(f.remaining(), 0.0);
    }
}
