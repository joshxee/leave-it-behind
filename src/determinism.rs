//! Makes runs reproducible: fixed RNG seed, fixed tick rate, and virtual time
//! advancing by exactly one fixed step per frame instead of wall-clock time
//! (so a slow SwiftShader frame doesn't change the simulation).
//!
//! Added by `GamePlugin` under the `e2e` feature; native tests add it
//! themselves via `tests/integration/common.rs`.

use std::time::Duration;

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;

use crate::{FIXED_HZ, GameRng};

pub const TEST_SEED: u64 = 0x1EA7_E17B;

pub fn fixed_step() -> Duration {
    Duration::from_secs_f64(1.0 / FIXED_HZ)
}

/// Number of `FixedUpdate` ticks run so far.
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct FixedTick(pub u64);

pub struct TestDeterminismPlugin;

impl Plugin for TestDeterminismPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameRng::from_seed(TEST_SEED))
            .insert_resource(Time::<Fixed>::from_duration(fixed_step()))
            .insert_resource(TimeUpdateStrategy::ManualDuration(fixed_step()))
            .init_resource::<FixedTick>()
            .add_systems(FixedFirst, count_tick);
    }
}

fn count_tick(mut tick: ResMut<FixedTick>) {
    tick.0 += 1;
}
