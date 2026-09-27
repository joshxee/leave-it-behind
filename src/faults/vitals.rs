//! The ship's vitals: oxygen and engine heat, one pool each, shared by
//! every fault of the kind that drains it. Hull breaches drain the oxygen,
//! loose bolts heat the engine. The flight is lost when the air runs out or
//! the engine overheats, however many faults it took.
//!
//! A fault's `clock` is its drain: alone and untouched, it takes `clock`
//! seconds to empty a full pool. Two at once drain twice as fast. The drain
//! falls as the repair goes on (each tight bolt, each second of tape). With
//! nothing draining it, a pool recovers from empty to full in
//! [`RECOVER_SECS`]. Trajectory drift has no pool: it keeps its own clock.

use bevy::prelude::*;

use super::{Fault, FaultKind};

/// Seconds for an empty pool to recover in full once nothing drains it.
pub const RECOVER_SECS: f32 = 30.0;
/// Floor on a drain, so time left is always finite.
const MIN_DRAIN: f32 = 1e-4;

/// One vital. `spent` 0 is full oxygen or a cool engine; 1 is fatal.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Pool {
    pub spent: f32,
    /// Spent per second by the faults draining it (0 while it recovers).
    pub drain: f32,
}

impl Pool {
    /// Drains by `drain` per second over `dt`, or recovers if nothing drains.
    pub fn step(&mut self, drain: f32, dt: f32) {
        self.drain = drain;
        self.spent = if drain > 0.0 {
            self.spent + drain * dt
        } else {
            self.spent - dt / RECOVER_SECS
        }
        .clamp(0.0, 1.0);
    }

    pub fn is_spent(&self) -> bool {
        self.spent >= 1.0
    }
}

#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
pub struct Vitals {
    pub oxygen: Pool,
    pub heat: Pool,
}

impl Vitals {
    /// The pool a kind of fault drains, if it has one.
    pub fn pool(&self, kind: FaultKind) -> Option<&Pool> {
        match kind {
            FaultKind::HullBreach => Some(&self.oxygen),
            FaultKind::LooseBolts => Some(&self.heat),
            FaultKind::TrajectoryDrift => None,
        }
    }

    pub fn pool_mut(&mut self, kind: FaultKind) -> Option<&mut Pool> {
        match kind {
            FaultKind::HullBreach => Some(&mut self.oxygen),
            FaultKind::LooseBolts => Some(&mut self.heat),
            FaultKind::TrajectoryDrift => None,
        }
    }

    /// Air left, 1 (full) to 0 (depleted).
    pub fn oxygen_left(&self) -> f32 {
        1.0 - self.oxygen.spent
    }

    /// Advances both pools by `dt` seconds for the faults on board.
    pub fn step<'a>(&mut self, faults: impl IntoIterator<Item = &'a Fault>, dt: f32) {
        let mut drain = [0.0; 2];
        for fault in faults {
            match fault.kind() {
                FaultKind::HullBreach => drain[0] += fault.drain(),
                FaultKind::LooseBolts => drain[1] += fault.drain(),
                FaultKind::TrajectoryDrift => {}
            }
        }
        self.oxygen.step(drain[0], dt);
        self.heat.step(drain[1], dt);
    }

    /// How close this fault's failure is, 0 to 1: its pool's `spent`, or
    /// its own clock for a kind with no pool.
    pub fn urgency(&self, fault: &Fault) -> f32 {
        self.pool(fault.kind())
            .map_or_else(|| fault.urgency(), |pool| pool.spent)
    }

    /// Seconds until this fault's failure at the current rate. A pool's
    /// drain counts at least this fault's own (a fault spawned this tick is
    /// not in the pool's drain yet).
    pub fn time_left(&self, fault: &Fault) -> f32 {
        self.pool(fault.kind()).map_or_else(
            || fault.remaining(),
            |pool| (1.0 - pool.spent) / pool.drain.max(fault.drain()).max(MIN_DRAIN),
        )
    }
}

pub(super) fn reset(mut vitals: ResMut<Vitals>) {
    *vitals = Vitals::default();
}

pub(super) fn drain_vitals(time: Res<Time>, faults: Query<&Fault>, mut vitals: ResMut<Vitals>) {
    vitals.step(faults.iter(), time.delta_secs());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::faults::Site;

    fn run(vitals: &mut Vitals, faults: &[Fault], secs: f32) {
        for _ in 0..(secs * 60.0).round() as usize {
            vitals.step(faults, 1.0 / 60.0);
        }
    }

    #[test]
    fn a_lone_fault_empties_its_pool_in_its_clock() {
        let mut v = Vitals::default();
        let breach = [Fault::new(Site::AirlockPortAft, 60.0)];
        run(&mut v, &breach, 30.0);
        assert!((v.oxygen_left() - 0.5).abs() < 0.01, "{v:?}");
        assert!((v.time_left(&breach[0]) - 30.0).abs() < 0.5);
        assert_eq!(v.heat, Pool::default(), "a breach never heats the engine");
        run(&mut v, &breach, 30.1);
        assert!(v.oxygen.is_spent());
    }

    #[test]
    fn two_breaches_drain_twice_as_fast() {
        let mut v = Vitals::default();
        let two = [
            Fault::new(Site::AirlockPortAft, 60.0),
            Fault::new(Site::HullStarboardMid, 60.0),
        ];
        run(&mut v, &two, 15.0);
        assert!((v.oxygen_left() - 0.5).abs() < 0.01, "{v:?}");
    }

    #[test]
    fn heat_rises_with_the_loose_bolts_left() {
        let mut v = Vitals::default();
        let mut bolts = Fault::new(Site::PortEngineInner, 60.0);
        bolts.repair = 2.0 / 3.0; // one bolt still loose
        run(&mut v, &[bolts], 30.0);
        assert!((v.heat.spent - 1.0 / 6.0).abs() < 0.01, "{v:?}");
    }

    #[test]
    fn pools_recover_once_nothing_drains_them() {
        let mut v = Vitals::default();
        v.oxygen.spent = 1.0;
        v.heat.spent = 0.4;
        run(&mut v, &[], RECOVER_SECS / 2.0);
        assert!((v.oxygen.spent - 0.5).abs() < 0.01, "{v:?}");
        assert_eq!(v.heat.spent, 0.0);
        assert_eq!(v.oxygen.drain, 0.0);
    }

    #[test]
    fn a_fault_on_a_spent_pool_is_as_urgent_as_the_pool() {
        let mut v = Vitals::default();
        v.oxygen.spent = 0.6;
        let fresh = Fault::new(Site::AirlockPortAft, 50.0);
        assert_eq!(v.urgency(&fresh), 0.6);
        // Not stepped yet: its own drain still counts.
        assert!((v.time_left(&fresh) - 20.0).abs() < 1e-3);
        let mut drift = Fault::new(Site::Helm, 40.0);
        drift.elapsed = 10.0;
        assert_eq!(v.urgency(&drift), 0.25);
        assert_eq!(v.time_left(&drift), 30.0);
    }
}
