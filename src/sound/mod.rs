//! Sparse ship audio. Door cues follow animation transitions; a short piece
//! of lab room tone occasionally surfaces during a flight.

use std::time::Duration;

use bevy::audio::{AudioSink, AudioSinkPlayback, Volume};
use bevy::prelude::*;

use crate::faults::drift::Nav;
use crate::faults::{Fault, FaultKind};
use crate::player::Player;
use crate::ship::CurrentRoom;
use crate::ship::RoomId;
use crate::ship::doors::DoorCue;
use crate::tools::ToolState;
use crate::{AppState, GameRng, GameSet, Pause, RunEntity, RunSet, running};

pub const CLANG_PATH: &str = "audio/clank-sound/Clank.wav";
pub const DOOR_PATH: &str = "audio/other-sounds/543404__alexo400__sci_fi_door.wav";
pub const LAB_PATH: &str = "audio/other-sounds/191852__vortex4__kaiju-lab-interior-ambience-1.wav";
pub const WRENCH_PATH: &str =
    "audio/gameplay/653305__monster_1999_kyle__car-wrench-sound-effect.wav";
pub const TAPE_PATH: &str = "audio/gameplay/565269__ninjasharkstudios__duct-tape_long_1.wav";
pub const STEERING_PATH: &str = "audio/gameplay/851916__tommasomotteran__derelict-spaceship-engine-rustbound-reactor-02-loop.wav";
pub const BREACH_PATH: &str = "audio/gameplay/244112__jobro__hull-breach.ogg";

// The supplied clang is sharp at its native level. These are linear gains.
const CLANG_GAIN: f32 = 0.12;
const DOOR_GAIN: f32 = 0.08;
const LAB_GAIN: f32 = 0.025;
const FIRST_AMBIENCE_SECS: f32 = 45.0;
const AMBIENCE_SECS: f32 = 8.0;
const FADE_SECS: f32 = 1.5;
const LOOP_FADE_SECS: f32 = 0.25;
const WRENCH_GAIN: f32 = 2.0; // The source recording itself is exceptionally quiet.
const TAPE_GAIN: f32 = 0.07;
const STEERING_GAIN: f32 = 0.12;
const BREACH_GAIN: f32 = 0.06;
const BREACH_REACH: f32 = 400.0;

#[derive(Resource)]
struct Sounds {
    clang: Handle<AudioSource>,
    door: Handle<AudioSource>,
    lab: Handle<AudioSource>,
    wrench: Handle<AudioSource>,
    tape: Handle<AudioSource>,
    steering: Handle<AudioSource>,
    breach: Handle<AudioSource>,
}

#[derive(Resource)]
struct AmbienceClock(f32);

#[derive(Component)]
struct ShipSound;

#[derive(Component)]
struct LabExcerpt;

#[derive(Component)]
struct WrenchSound(Entity);

#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum LoopKind {
    Tape,
    Steering,
    Breach,
}

impl LoopKind {
    const ALL: [Self; 3] = [Self::Tape, Self::Steering, Self::Breach];

    fn index(self) -> usize {
        self as usize
    }
}

#[derive(Component)]
struct LoopSound {
    kind: LoopKind,
    gain: f32,
}

pub struct SoundPlugin;

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut App) {
        // MinimalPlugins tests have no AssetServer or audio output.
        if let Some(server) = app.world().get_resource::<AssetServer>() {
            app.insert_resource(Sounds {
                clang: server.load(CLANG_PATH),
                door: server.load(DOOR_PATH),
                lab: server.load(LAB_PATH),
                wrench: server.load(WRENCH_PATH),
                tape: server.load(TAPE_PATH),
                steering: server.load(STEERING_PATH),
                breach: server.load(BREACH_PATH),
            });
        }
        app.insert_resource(AmbienceClock(FIRST_AMBIENCE_SECS))
            .add_systems(
                OnEnter(AppState::Playing),
                reset_ambience.in_set(RunSet::Spawn),
            )
            .add_systems(
                FixedUpdate,
                (play_door_cues, tick_ambience, fade_ambience)
                    .in_set(GameSet::Act)
                    .run_if(running),
            )
            .add_systems(
                FixedUpdate,
                (sync_wrench, sync_loops)
                    .in_set(GameSet::Present)
                    .run_if(running),
            )
            .add_systems(OnEnter(Pause::Paused), pause_sounds)
            .add_systems(Update, pause_sounds.run_if(in_state(Pause::Paused)))
            .add_systems(OnExit(Pause::Paused), resume_sounds)
            .add_systems(OnEnter(AppState::Landed), stop_sounds)
            .add_systems(OnEnter(AppState::Lost), stop_sounds);
    }
}

fn reset_ambience(mut clock: ResMut<AmbienceClock>) {
    clock.0 = FIRST_AMBIENCE_SECS;
}

fn play_door_cues(
    mut commands: Commands,
    mut cues: MessageReader<DoorCue>,
    sounds: Option<Res<Sounds>>,
) {
    let Some(sounds) = sounds else {
        // Drain cues in headless tests too.
        let _ = cues.read().count();
        return;
    };
    for cue in cues.read() {
        let (handle, gain) = match cue {
            DoorCue::Opening => (&sounds.door, DOOR_GAIN),
            DoorCue::Closed => (&sounds.clang, CLANG_GAIN),
        };
        commands.spawn((
            Name::new("Door sound"),
            ShipSound,
            RunEntity,
            AudioPlayer::new(handle.clone()),
            PlaybackSettings::DESPAWN.with_volume(Volume::Linear(gain)),
        ));
    }
}

fn tick_ambience(
    time: Res<Time>,
    mut commands: Commands,
    mut clock: ResMut<AmbienceClock>,
    mut rng: ResMut<GameRng>,
    sounds: Option<Res<Sounds>>,
) {
    clock.0 -= time.delta_secs();
    if clock.0 > 0.0 {
        return;
    }
    if let Some(sounds) = sounds {
        // The source is a minute long; borrow only a brief, unpredictable
        // stretch, leaving a long interval of silence between visits.
        let offset = rng.range(0.0, 45.0);
        commands.spawn((
            Name::new("Distant ship ambience"),
            ShipSound,
            LabExcerpt,
            RunEntity,
            AudioPlayer::new(sounds.lab.clone()),
            PlaybackSettings::DESPAWN
                .with_volume(Volume::SILENT)
                .with_start_position(Duration::from_secs_f32(offset))
                .with_duration(Duration::from_secs_f32(AMBIENCE_SECS)),
        ));
    }
    clock.0 = rng.range(95.0, 145.0);
}

fn fade_ambience(mut sinks: Query<&mut AudioSink, With<LabExcerpt>>) {
    for mut sink in &mut sinks {
        let played = sink.position().as_secs_f32();
        let fade_in = (played / FADE_SECS).clamp(0.0, 1.0);
        let fade_out = ((AMBIENCE_SECS - played) / FADE_SECS).clamp(0.0, 1.0);
        sink.set_volume(Volume::Linear(LAB_GAIN * fade_in.min(fade_out)));
    }
}

/// A valid bolt turn plays one short part of the long source recording.
/// The entity stays until the turn ends so a completed clip cannot retrigger.
fn sync_wrench(
    mut commands: Commands,
    tools: Res<ToolState>,
    active: Query<(Entity, &WrenchSound)>,
    sounds: Option<Res<Sounds>>,
) {
    let target = tools.turn.map(|turn| turn.target);
    let mut matching = false;
    for (entity, sound) in &active {
        if Some(sound.0) == target {
            matching = true;
        } else {
            commands.entity(entity).despawn();
        }
    }
    if let (Some(target), false, Some(sounds)) = (target, matching, sounds) {
        commands.spawn((
            Name::new("Wrench turn"),
            ShipSound,
            WrenchSound(target),
            RunEntity,
            AudioPlayer::new(sounds.wrench.clone()),
            PlaybackSettings::ONCE
                .with_volume(Volume::Linear(WRENCH_GAIN))
                .with_start_position(Duration::from_millis(600))
                .with_duration(Duration::from_millis(850)),
        ));
    }
}

/// The nearest open breach is only audible in its room. Tape and steering
/// have no sound when the corresponding action is idle.
fn breach_gain(
    listener: Option<(Vec2, RoomId)>,
    breaches: impl IntoIterator<Item = (Vec2, RoomId, f32)>,
) -> f32 {
    let Some((position, room)) = listener else {
        return 0.0;
    };
    breaches
        .into_iter()
        .filter(|(_, site_room, repair)| *site_room == room && *repair < 1.0)
        .map(|(site, _, repair)| {
            let near = (1.0 - position.distance(site) / BREACH_REACH).clamp(0.0, 1.0);
            BREACH_GAIN * near * (1.0 - repair)
        })
        .fold(0.0, f32::max)
}

fn sync_loops(
    mut commands: Commands,
    time: Res<Time>,
    tools: Res<ToolState>,
    nav: Res<Nav>,
    room: Res<CurrentRoom>,
    players: Query<&Transform, With<Player>>,
    faults: Query<&Fault>,
    mut active: Query<(Entity, &mut LoopSound, Option<&mut AudioSink>)>,
    sounds: Option<Res<Sounds>>,
) {
    let listener = players
        .iter()
        .next()
        .map(|p| (p.translation.truncate(), room.0));
    let breach = breach_gain(
        listener,
        faults
            .iter()
            .filter(|f| f.kind() == FaultKind::HullBreach)
            .map(|f| (f.site.pos(), f.site.room(), f.repair)),
    );
    let wanted = [
        if tools.taping.is_some() {
            TAPE_GAIN
        } else {
            0.0
        },
        if nav.engaged && nav.stick.length_squared() > 0.0 {
            STEERING_GAIN
        } else {
            0.0
        },
        breach,
    ];
    let mut found = [false; 3];
    for (entity, mut loop_sound, sink) in &mut active {
        let index = loop_sound.kind.index();
        found[index] = true;
        let target = wanted[index];
        let step =
            time.delta_secs() * [TAPE_GAIN, STEERING_GAIN, BREACH_GAIN][index] / LOOP_FADE_SECS;
        loop_sound.gain += (target - loop_sound.gain).clamp(-step, step);
        if let Some(mut sink) = sink {
            sink.set_volume(Volume::Linear(loop_sound.gain));
        }
        if target == 0.0 && loop_sound.gain <= 0.001 {
            commands.entity(entity).despawn();
        }
    }
    let Some(sounds) = sounds else { return };
    for kind in LoopKind::ALL {
        let index = kind.index();
        if found[index] || wanted[index] <= 0.0 {
            continue;
        }
        let (name, handle) = match kind {
            LoopKind::Tape => ("Tape feed", &sounds.tape),
            LoopKind::Steering => ("Steering engine", &sounds.steering),
            LoopKind::Breach => ("Nearby hull breach", &sounds.breach),
        };
        commands.spawn((
            Name::new(name),
            ShipSound,
            LoopSound { kind, gain: 0.0 },
            RunEntity,
            AudioPlayer::new(handle.clone()),
            PlaybackSettings::LOOP.with_volume(Volume::SILENT),
        ));
    }
}

fn pause_sounds(sinks: Query<&AudioSink, With<ShipSound>>) {
    for sink in &sinks {
        sink.pause();
    }
}

fn resume_sounds(sinks: Query<&AudioSink, With<ShipSound>>) {
    for sink in &sinks {
        sink.play();
    }
}

fn stop_sounds(mut commands: Commands, sounds: Query<Entity, With<ShipSound>>) {
    for entity in &sounds {
        commands.entity(entity).despawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::faults::Site;
    use crate::ship::layout::WallContact;
    use crate::tools::WrenchTurn;
    use bevy::audio::Decodable;
    use bevy::ecs::system::RunSystemOnce;

    fn dummy_sounds() -> Sounds {
        Sounds {
            clang: Handle::default(),
            door: Handle::default(),
            lab: Handle::default(),
            wrench: Handle::default(),
            tape: Handle::default(),
            steering: Handle::default(),
            breach: Handle::default(),
        }
    }

    #[test]
    fn shipped_audio_decodes_with_the_enabled_audio_features() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
        for path in [
            CLANG_PATH,
            DOOR_PATH,
            LAB_PATH,
            WRENCH_PATH,
            TAPE_PATH,
            STEERING_PATH,
            BREACH_PATH,
        ] {
            let bytes = std::fs::read(root.join(path)).unwrap();
            let source = AudioSource {
                bytes: bytes.into(),
            };
            assert!(source.decoder().next().is_some(), "{path}");
        }
    }

    #[test]
    fn breach_sound_stays_local_and_fades_as_it_is_repaired() {
        let room = RoomId::Quarters;
        let other_room = RoomId::Airlock;
        let listener = Some((Vec2::ZERO, room));
        let near = breach_gain(listener, [(Vec2::ZERO, room, 0.0)]);
        assert_eq!(near, BREACH_GAIN);
        assert_eq!(breach_gain(listener, [(Vec2::ZERO, other_room, 0.0)]), 0.0);
        assert_eq!(breach_gain(listener, [(Vec2::ZERO, room, 1.0)]), 0.0);
        assert_eq!(
            breach_gain(listener, [(Vec2::X * BREACH_REACH, room, 0.0)]),
            0.0
        );
        assert!(breach_gain(listener, [(Vec2::ZERO, room, 0.5)]) < near);
    }

    #[test]
    fn wrench_sound_starts_once_per_turn_and_stops_on_cancel() {
        let mut world = World::new();
        world.insert_resource(dummy_sounds());
        let target = world.spawn_empty().id();
        let mut tools = ToolState::default();
        tools.turn = Some(WrenchTurn { target, left: 0.9 });
        world.insert_resource(tools);
        world.run_system_once(sync_wrench).unwrap();
        world.run_system_once(sync_wrench).unwrap();
        let mut active = world.query::<(&WrenchSound, &PlaybackSettings)>();
        let turns: Vec<_> = active.iter(&world).collect();
        assert_eq!(turns.len(), 1);
        let (sound, settings) = turns[0];
        assert_eq!(sound.0, target);
        assert_eq!(settings.start_position, Some(Duration::from_millis(600)));
        world.resource_mut::<ToolState>().turn = None;
        world.run_system_once(sync_wrench).unwrap();
        assert_eq!(active.iter(&world).count(), 0);
    }

    #[test]
    fn tape_steering_and_nearby_breach_loops_follow_active_actions() {
        let mut world = World::new();
        world.insert_resource(dummy_sounds());
        let mut time = Time::<()>::default();
        time.advance_by(Duration::from_secs(1));
        world.insert_resource(time);
        let site = Site::AirlockPortAft;
        world.insert_resource(CurrentRoom(site.room()));
        let mut tools = ToolState::default();
        tools.taping = Some(WallContact {
            point: site.pos(),
            normal: site.normal(),
        });
        world.insert_resource(tools);
        world.insert_resource(Nav {
            engaged: true,
            stick: Vec2::X,
            ..default()
        });
        world.spawn((Player, Transform::from_translation(site.pos().extend(0.0))));
        let fault = world.spawn(Fault::new(site, 60.0)).id();
        world.run_system_once(sync_loops).unwrap();
        let mut active = world.query::<&LoopSound>();
        assert_eq!(active.iter(&world).count(), 3);

        world.resource_mut::<ToolState>().taping = None;
        world.resource_mut::<Nav>().engaged = false;
        world.get_mut::<Fault>(fault).unwrap().repair = 1.0;
        world.run_system_once(sync_loops).unwrap();
        assert_eq!(active.iter(&world).count(), 0);
    }
}
