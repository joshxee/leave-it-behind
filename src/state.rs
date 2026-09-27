//! Top-level app state and the system-set ordering every feature plugs into.

use bevy::prelude::*;

use crate::scenarios::ActiveScenario;

#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppState {
    /// First frame(s). A future loading screen lives here.
    #[default]
    Boot,
    /// The title card, the main menu and its screens (see `menu`).
    Menu,
    /// The flight: faults and repairs until every fault is fixed. Pausing is
    /// the [`Pause`] sub-state, so it never restarts the run.
    Playing,
    /// Every fault fixed and the ship touched down. Victory.
    Landed,
    /// A fault's failure clock ran out.
    Lost,
}

impl AppState {
    pub fn as_str(self) -> &'static str {
        match self {
            AppState::Boot => "Boot",
            AppState::Menu => "Menu",
            AppState::Playing => "Playing",
            AppState::Landed => "Landed",
            AppState::Lost => "Lost",
        }
    }
}

/// Paused or not, while [`AppState::Playing`]. Entering `Playing` starts a
/// fresh run, so pausing lives here instead of beside it in [`AppState`].
/// A restart (`Playing` again) keeps the current value: set `Running` with it.
#[derive(SubStates, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[source(AppState = AppState::Playing)]
pub enum Pause {
    #[default]
    Running,
    Paused,
}

impl Pause {
    pub fn as_str(self) -> &'static str {
        match self {
            Pause::Running => "Running",
            Pause::Paused => "Paused",
        }
    }
}

/// Run condition for gameplay: a flight is on and not paused.
pub fn running(pause: Option<Res<State<Pause>>>) -> bool {
    pause.is_some_and(|p| *p.get() == Pause::Running)
}

/// Run condition for in-world animation: true except during a paused
/// flight. (Animations read `Time`, which keeps going while paused.)
pub fn not_paused(pause: Option<Res<State<Pause>>>) -> bool {
    !pause.is_some_and(|p| *p.get() == Pause::Paused)
}

/// Ordering for game systems. Configured (chained) in `Startup`, `Update`
/// and `FixedUpdate`, so every feature slots into the same pipeline.
///
/// `Update` uses `Input` and `Present` (menus also use `Act` for their
/// actions); `FixedUpdate` uses `Move` through `Resolve`; `Startup` spawns
/// in `Input`.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameSet {
    /// Spawning entities (Startup) / reading raw input into intents (Update).
    Input,
    /// Player facing and movement, current room.
    Move,
    /// Tools and interactions: wrench, tape, helm, diagnostics.
    Act,
    /// Fault clocks and repairs, the fault schedule, the journey clock.
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
/// in [`RunSet::Cleanup`] when a new run starts, and on the way to the menu.
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct RunEntity;

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>().add_sub_state::<Pause>();
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
            )
            .add_systems(OnEnter(AppState::Menu), despawn_run_entities);
    }
}

/// Nothing to load yet. A scenario (tests, e2e fixtures) starts a flight
/// straight away; players get the title screen.
fn finish_boot(scenario: Option<Res<ActiveScenario>>, mut next: ResMut<NextState<AppState>>) {
    next.set(if scenario.is_some() {
        AppState::Playing
    } else {
        AppState::Menu
    });
}

fn despawn_run_entities(mut commands: Commands, entities: Query<Entity, With<RunEntity>>) {
    for entity in &entities {
        // A tagged child may already be gone with its tagged parent.
        commands.entity(entity).try_despawn();
    }
}
