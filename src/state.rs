//! Top-level app state and the system-set ordering every feature plugs into.

use bevy::prelude::*;

#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppState {
    /// First frame(s). A future loading screen lives here.
    #[default]
    Boot,
    /// The flight: faults, repairs and the countdown to arrival.
    Playing,
    /// The countdown reached zero and the ship touched down. Victory.
    Landed,
    /// A fault's failure clock ran out.
    Lost,
}

impl AppState {
    pub fn as_str(self) -> &'static str {
        match self {
            AppState::Boot => "Boot",
            AppState::Playing => "Playing",
            AppState::Landed => "Landed",
            AppState::Lost => "Lost",
        }
    }
}

/// Ordering for game systems. Configured (chained) in `Startup`, `Update`
/// and `FixedUpdate`, so every feature slots into the same pipeline.
///
/// `Update` uses `Input` and `Present`; `FixedUpdate` uses `Move` through
/// `Resolve`; `Startup` spawns in `Input`.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameSet {
    /// Spawning entities (Startup) / reading raw input into intents (Update).
    Input,
    /// Player facing and movement, current room.
    Move,
    /// Tools and interactions: wrench, tape, helm, diagnostics.
    Act,
    /// Fault clocks and repairs, the fault schedule, the journey countdown.
    Simulate,
    /// Fixed and failed faults, and the flight's outcome.
    Resolve,
    /// Presentation: sprites, camera, HUD.
    Present,
}

/// Ordering inside `OnEnter(AppState::Playing)`, which runs on launch and on
/// every restart: clear the previous run, start a fresh one, then apply the
/// active scenario (if any) on top.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RunSet {
    Cleanup,
    Spawn,
    Scenario,
}

/// Marks entities that belong to one run (player, faults, tape). Despawned
/// in [`RunSet::Cleanup`] when a new run starts.
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct RunEntity;

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>();
        let order = || {
            (
                GameSet::Input,
                GameSet::Move,
                GameSet::Act,
                GameSet::Simulate,
                GameSet::Resolve,
                GameSet::Present,
            )
                .chain()
        };
        app.configure_sets(Startup, order())
            .configure_sets(Update, order())
            .configure_sets(FixedUpdate, order())
            .configure_sets(
                OnEnter(AppState::Playing),
                (RunSet::Cleanup, RunSet::Spawn, RunSet::Scenario).chain(),
            )
            .add_systems(Update, finish_boot.run_if(in_state(AppState::Boot)))
            .add_systems(
                OnEnter(AppState::Playing),
                despawn_run_entities.in_set(RunSet::Cleanup),
            );
    }
}

/// Nothing to load yet: go straight to gameplay.
fn finish_boot(mut next: ResMut<NextState<AppState>>) {
    next.set(AppState::Playing);
}

fn despawn_run_entities(mut commands: Commands, entities: Query<Entity, With<RunEntity>>) {
    for entity in &entities {
        // A tagged child may already be gone with its tagged parent.
        commands.entity(entity).try_despawn();
    }
}
