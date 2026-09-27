//! Windows-friendly native render verification, using the real game and repair systems.
//! cargo run --example maintenance_smoke -- bolts (also drift, breach, quiet)
use bevy::{
    asset::AssetMetaCheck,
    prelude::*,
    render::view::screenshot::{Screenshot, save_to_disk},
};
use leave_it_behind::{
    ActiveScenario, AppState, GamePlugin, GameSet, Scenario, TestDeterminismPlugin, primary_window,
};
use leave_it_behind::{
    faults::{Fault, Site, drift::Nav},
    level::RunStats,
    player::{Player, PlayerIntent},
    tools::{Tool, ToolBelt},
};

#[derive(Resource)]
struct Capture {
    scenario: Scenario,
    frame: usize,
}

fn main() {
    let name = std::env::args().nth(1).unwrap_or_else(|| "quiet".into());
    let scenario = Scenario::from_name(&name).expect("scenario name");
    std::fs::create_dir_all("test-reports/maintenance").unwrap();
    let mut app = App::new();
    let mut window = primary_window();
    window.visible = false;
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(window),
                ..default()
            })
            .set(ImagePlugin::default_nearest())
            .set(AssetPlugin {
                meta_check: AssetMetaCheck::Never,
                ..default()
            }),
    )
    .insert_resource(leave_it_behind::save::Storage::memory())
    .add_plugins(GamePlugin);
    if !app.is_plugin_added::<TestDeterminismPlugin>() {
        app.add_plugins(TestDeterminismPlugin);
    }
    app.insert_resource(ActiveScenario(scenario))
        .insert_resource(Capture { scenario, frame: 0 })
        .add_systems(
            Update,
            capture
                .after(GameSet::Input)
                .before(GameSet::Present)
                .run_if(in_state(AppState::Playing)),
        )
        .run();
}

fn capture(
    mut commands: Commands,
    mut capture: ResMut<Capture>,
    mut intent: ResMut<PlayerIntent>,
    nav: Res<Nav>,
    mut belt: ResMut<ToolBelt>,
    mut players: Query<&mut Transform, With<Player>>,
    faults: Query<&Fault>,
    stats: Res<RunStats>,
    mut exit: MessageWriter<AppExit>,
) {
    capture.frame += 1;
    let f = capture.frame;
    let scenario = capture.scenario;
    intent.aim = None;
    intent.direction = Vec2::ZERO;
    intent.use_held = false;
    let mut shot = None;
    if f == 180 {
        shot = Some("before");
    }
    match scenario {
        Scenario::Bolts => {
            let index = ((f.saturating_sub(200)) / 90).min(2);
            let bolt = Site::PortEngineInner.bolts().unwrap()[index];
            if f >= 200 {
                let mut player = players.single_mut().unwrap();
                player.translation = (leave_it_behind::scenarios::bolts_stand() + bolt
                    - Site::PortEngineInner.bolts().unwrap()[0])
                    .extend(3.0);
                intent.aim = Some(bolt);
                if (f - 200).is_multiple_of(90) && f <= 380 {
                    intent.use_presses = 1;
                }
            }
            if f == 220 {
                shot = Some("turning");
            }
            if f == 440 {
                shot = Some("repaired");
            }
        }
        Scenario::Breach => {
            belt.held = Tool::Tape;
            intent.aim = Some(Site::AirlockPortAft.pos());
            intent.use_held = (200..390).contains(&f);
            if f == 250 {
                shot = Some("applying");
            }
            if f == 440 {
                shot = Some("sealed");
            }
        }
        Scenario::Drift => {
            if f == 200 {
                intent.interact_presses = 1;
            }
            if f > 200 && !faults.is_empty() {
                intent.direction = Vec2::new(
                    if nav.marker.x.abs() > 0.04 {
                        -nav.marker.x.signum()
                    } else {
                        0.0
                    },
                    if nav.marker.y.abs() > 0.04 {
                        -nav.marker.y.signum()
                    } else {
                        0.0
                    },
                );
            }
            if f == 320 {
                shot = Some("centering");
            }
            if f == 560 {
                shot = Some("locked");
            }
        }
        Scenario::Quiet => {
            let room = leave_it_behind::ship::RoomId::Quarters.interior();
            if f >= 200 {
                let p = Vec2::new(
                    room.center().x - 100.0,
                    room.min.y + leave_it_behind::player::PLAYER_RADIUS,
                );
                players.single_mut().unwrap().translation = p.extend(3.0);
                intent.aim = Some(p + Vec2::NEG_Y * 100.0);
            }
            if f == 250 {
                shot = Some("foreground");
            }
        }
        _ => {}
    }
    if let Some(label) = shot {
        let path = format!("test-reports/maintenance/{}-{label}.png", scenario.name());
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path));
    }
    if f == 620 {
        if matches!(
            scenario,
            Scenario::Bolts | Scenario::Breach | Scenario::Drift
        ) {
            assert_eq!(stats.fixed, 1, "native repair did not finish");
            assert!(faults.is_empty());
        }
        info!(
            "MAINTENANCE_RENDER_PASS: {} fixed={}",
            scenario.name(),
            stats.fixed
        );
        exit.write(AppExit::Success);
    }
}
