//! Plays a clank when the score changes. Skipped entirely when there is no
//! `AssetServer` (headless tests with `MinimalPlugins`).

use bevy::prelude::*;

use crate::GameSet;
use crate::scoring::ScoreChanged;

pub const CLANK_PATH: &str = "audio/clank-sound/Clank.wav";

#[derive(Resource)]
struct Sfx {
    clank: Handle<AudioSource>,
}

pub struct AudioFxPlugin;

impl Plugin for AudioFxPlugin {
    fn build(&self, app: &mut App) {
        if !app.world().contains_resource::<AssetServer>() {
            return;
        }
        app.add_systems(Startup, load_sfx)
            .add_systems(Update, play_clank.in_set(GameSet::Present));
    }
}

fn load_sfx(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(Sfx {
        clank: asset_server.load(CLANK_PATH),
    });
}

fn play_clank(mut commands: Commands, sfx: Res<Sfx>, mut changed: MessageReader<ScoreChanged>) {
    if changed.read().count() > 0 {
        commands.spawn((AudioPlayer(sfx.clank.clone()), PlaybackSettings::DESPAWN));
    }
}
