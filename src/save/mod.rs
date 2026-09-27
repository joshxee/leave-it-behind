//! Saving: [`Settings`] and [`Progress`], loaded at startup and written
//! whenever they change (checked every frame in `Last`, so the frame that
//! quits saves too). There is no save button and there are no slots.
//!
//! - Desktop: `settings.ron` and `save.ron` in the OS app-data folder
//!   ([`data_dir`]), each written to a temp file and renamed into place.
//! - Web: `localStorage`, under `leave-it-behind/<key>`.
//! - Tests and native `e2e` builds: in memory, so they never touch a real save.
//!
//! Files carry a format `version`. A file that cannot be read (corrupt, or
//! from a newer version of the game) is copied to `<key>.bak`, the game
//! starts from defaults, and a notice says so. It never crashes.

#[cfg(not(target_arch = "wasm32"))]
use std::path::{Path, PathBuf};

use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::level::Progress;
use crate::settings::Settings;
use crate::ui::Notices;

pub const SETTINGS_KEY: &str = "settings";
pub const SAVE_KEY: &str = "save";
/// Current save format. Bump it (and migrate in `decode`) only for changes
/// that old files cannot be read into: added fields just take defaults.
pub const FORMAT_VERSION: u32 = 1;
/// Folder name (desktop) and key prefix (web).
pub const GAME_DIR: &str = "leave-it-behind";

/// Where saves live. Insert one before `GamePlugin` to override the
/// platform default (tests insert [`Storage::memory`]).
#[derive(Resource, Debug)]
pub struct Storage(Backend);

#[derive(Debug)]
enum Backend {
    Memory(HashMap<String, String>),
    #[cfg(not(target_arch = "wasm32"))]
    Dir(PathBuf),
    #[cfg(target_arch = "wasm32")]
    Local,
    /// Nowhere to save (no home folder): the game runs, nothing persists.
    Nowhere,
}

impl Storage {
    pub fn memory() -> Self {
        Self(Backend::Memory(HashMap::default()))
    }

    /// Saves go nowhere: the game runs, nothing persists.
    pub fn disabled() -> Self {
        Self(Backend::Nowhere)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn in_dir(dir: impl Into<PathBuf>) -> Self {
        Self(Backend::Dir(dir.into()))
    }

    /// The app-data folder ([`data_dir`]).
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "e2e")))]
    pub fn platform_default() -> Self {
        data_dir(std::env::consts::OS, |k| std::env::var(k).ok())
            .map_or_else(Self::disabled, Self::in_dir)
    }

    /// Native `e2e` builds keep saves in memory: scenario runs must never
    /// overwrite a player's real progress.
    #[cfg(all(not(target_arch = "wasm32"), feature = "e2e"))]
    pub fn platform_default() -> Self {
        Self::memory()
    }

    /// `localStorage`.
    #[cfg(target_arch = "wasm32")]
    pub fn platform_default() -> Self {
        Self(Backend::Local)
    }

    pub fn read(&self, key: &str) -> Result<Option<String>, String> {
        match &self.0 {
            Backend::Memory(map) => Ok(map.get(key).cloned()),
            #[cfg(not(target_arch = "wasm32"))]
            Backend::Dir(dir) => match std::fs::read_to_string(file_path(dir, key)) {
                Ok(text) => Ok(Some(text)),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
                Err(e) => Err(e.to_string()),
            },
            #[cfg(target_arch = "wasm32")]
            Backend::Local => local_storage()?
                .get_item(&web_key(key))
                .map_err(|_| "localStorage read failed".to_string()),
            Backend::Nowhere => Ok(None),
        }
    }

    pub fn write(&mut self, key: &str, text: &str) -> Result<(), String> {
        match &mut self.0 {
            Backend::Memory(map) => {
                map.insert(key.to_string(), text.to_string());
                Ok(())
            }
            #[cfg(not(target_arch = "wasm32"))]
            Backend::Dir(dir) => write_atomically(dir, key, text).map_err(|e| e.to_string()),
            #[cfg(target_arch = "wasm32")]
            Backend::Local => local_storage()?
                .set_item(&web_key(key), text)
                .map_err(|_| "localStorage is full or blocked".to_string()),
            Backend::Nowhere => Ok(()),
        }
    }
}

/// The folder saves go in: `%APPDATA%\leave-it-behind` on Windows,
/// `~/Library/Application Support/leave-it-behind` on macOS, and
/// `$XDG_DATA_HOME/leave-it-behind` (default `~/.local/share`) elsewhere.
/// `None` without the variables that locate it.
#[cfg(not(target_arch = "wasm32"))]
pub fn data_dir(os: &str, env: impl Fn(&str) -> Option<String>) -> Option<PathBuf> {
    let var = |key: &str| env(key).filter(|v| !v.is_empty()).map(PathBuf::from);
    let base = match os {
        "windows" => var("APPDATA")?,
        "macos" => var("HOME")?.join("Library").join("Application Support"),
        _ => var("XDG_DATA_HOME")
            // XDG uses Unix paths, including when testing this branch on Windows.
            .filter(|p| p.to_string_lossy().starts_with('/'))
            .or_else(|| Some(var("HOME")?.join(".local").join("share")))?,
    };
    Some(base.join(GAME_DIR))
}

#[cfg(not(target_arch = "wasm32"))]
fn file_path(dir: &Path, key: &str) -> PathBuf {
    dir.join(format!("{key}.ron"))
}

/// A crash or power cut mid-write leaves the old file, never half a file.
#[cfg(not(target_arch = "wasm32"))]
fn write_atomically(dir: &Path, key: &str, text: &str) -> std::io::Result<()> {
    use std::io::Write;
    std::fs::create_dir_all(dir)?;
    let tmp = dir.join(format!("{key}.ron.tmp"));
    let mut file = std::fs::File::create(&tmp)?;
    file.write_all(text.as_bytes())?;
    file.sync_all()?;
    drop(file);
    std::fs::rename(&tmp, file_path(dir, key))
}

#[cfg(target_arch = "wasm32")]
fn web_key(key: &str) -> String {
    format!("{GAME_DIR}/{key}")
}

#[cfg(target_arch = "wasm32")]
fn local_storage() -> Result<web_sys::Storage, String> {
    web_sys::window()
        .ok_or("no window")?
        .local_storage()
        .map_err(|_| "localStorage is blocked")?
        .ok_or_else(|| "no localStorage".to_string())
}

#[derive(Serialize, Deserialize)]
struct SettingsFile {
    version: u32,
    settings: Settings,
}

#[derive(Serialize, Deserialize)]
struct SaveFile {
    version: u32,
    progress: Progress,
}

#[derive(Deserialize)]
struct VersionOnly {
    version: u32,
}

fn pretty() -> ron::ser::PrettyConfig {
    ron::ser::PrettyConfig::default()
}

pub fn encode_settings(settings: &Settings) -> String {
    let file = SettingsFile {
        version: FORMAT_VERSION,
        settings: settings.clone(),
    };
    ron::ser::to_string_pretty(&file, pretty()).expect("settings serialize")
}

pub fn encode_progress(progress: &Progress) -> String {
    let file = SaveFile {
        version: FORMAT_VERSION,
        progress: progress.clone(),
    };
    ron::ser::to_string_pretty(&file, pretty()).expect("progress serialize")
}

/// Checks the version first, so a newer game's file is reported as such
/// instead of as garbage.
fn decode<F: DeserializeOwned>(text: &str) -> Result<F, String> {
    let version = ron::from_str::<VersionOnly>(text)
        .map_err(|e| format!("unreadable ({e})"))?
        .version;
    if version > FORMAT_VERSION {
        return Err(format!("saved by a newer version (format {version})"));
    }
    ron::from_str(text).map_err(|e| format!("unreadable ({e})"))
}

pub fn decode_settings(text: &str) -> Result<Settings, String> {
    decode::<SettingsFile>(text).map(|f| f.settings)
}

pub fn decode_progress(text: &str) -> Result<Progress, String> {
    decode::<SaveFile>(text).map(|f| f.progress)
}

/// Reads one key. Anything unreadable is backed up to `<key>.bak` and
/// replaced by defaults; the notice says what happened.
pub fn load<T: Default>(
    storage: &mut Storage,
    key: &str,
    what: &str,
    decode: impl Fn(&str) -> Result<T, String>,
) -> (T, Option<String>) {
    let text = match storage.read(key) {
        Ok(Some(text)) => text,
        Ok(None) => return (T::default(), None),
        Err(e) => {
            warn!("could not read {what}: {e}");
            return (
                T::default(),
                Some(format!("Could not read {what}. Using defaults.")),
            );
        }
    };
    match decode(&text) {
        Ok(value) => (value, None),
        Err(e) => {
            warn!("{what} {e}; starting from defaults");
            let backup = format!("{key}.bak");
            let kept = storage.write(&backup, &text).is_ok();
            let notice = if kept {
                format!("Could not read {what}. Using defaults (old copy kept as {backup}).")
            } else {
                format!("Could not read {what}. Using defaults.")
            };
            (T::default(), Some(notice))
        }
    }
}

/// What was last read or written, so only real changes are saved.
#[derive(Resource, Debug, Clone, PartialEq)]
struct Persisted {
    settings: Settings,
    progress: Progress,
}

pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        if !app.world().contains_resource::<Storage>() {
            app.insert_resource(Storage::platform_default());
        }
        app.init_resource::<Notices>();
        let world = app.world_mut();
        let mut storage = world.resource_mut::<Storage>();
        let (settings, settings_notice) =
            load(&mut storage, SETTINGS_KEY, "settings", decode_settings);
        let (progress, progress_notice) = load(&mut storage, SAVE_KEY, "progress", decode_progress);
        let mut notices = world.resource_mut::<Notices>();
        for notice in [settings_notice, progress_notice].into_iter().flatten() {
            notices.push(notice);
        }
        app.insert_resource(Persisted {
            settings: settings.clone(),
            progress: progress.clone(),
        })
        .insert_resource(settings)
        .insert_resource(progress)
        .add_systems(Last, persist);
    }
}

/// Writes settings and progress when they differ from what was last saved.
fn persist(
    settings: Res<Settings>,
    progress: Res<Progress>,
    mut saved: ResMut<Persisted>,
    mut storage: ResMut<Storage>,
    mut notices: ResMut<Notices>,
) {
    if settings.is_changed() && *settings != saved.settings {
        saved.settings = settings.clone();
        if let Err(e) = storage.write(SETTINGS_KEY, &encode_settings(&settings)) {
            warn!("could not save settings: {e}");
            notices.push("Could not save settings.");
        }
    }
    if progress.is_changed() && *progress != saved.progress {
        saved.progress = progress.clone();
        if let Err(e) = storage.write(SAVE_KEY, &encode_progress(&progress)) {
            warn!("could not save progress: {e}");
            notices.push("Could not save progress.");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coach::{Tip, TipsSeen};
    use crate::level::{Damage, RunRecord};

    fn some_progress() -> Progress {
        let mut p = Progress::default();
        p.record(
            "one",
            RunRecord {
                landed: true,
                damage: Damage {
                    oxygen: 12.5,
                    course: 3.0,
                    engine: 0.25,
                },
                survived: 240.0,
            },
        );
        p
    }

    #[test]
    fn settings_and_progress_round_trip() {
        let mut tips_seen = TipsSeen::default();
        tips_seen.mark(Tip::Preflight);
        tips_seen.mark(Tip::HullBreach);
        let settings = Settings {
            shake: 25,
            flash: 0,
            controls_hint: false,
            tips_seen,
            ..default()
        };
        assert_eq!(decode_settings(&encode_settings(&settings)), Ok(settings));
        let progress = some_progress();
        assert_eq!(decode_progress(&encode_progress(&progress)), Ok(progress));
    }

    #[test]
    fn missing_fields_take_defaults() {
        let s = decode_settings("(version: 1, settings: (shake: 50))").unwrap();
        assert_eq!(s.shake, 50);
        assert_eq!(s.flash, Settings::default().flash);
        // Saved before the coaching existed: every tip is still to come.
        assert_eq!(s.tips_seen, TipsSeen::default());
        assert_eq!(
            decode_progress("(version: 1, progress: ())"),
            Ok(Progress::default())
        );
    }

    #[test]
    fn garbage_and_newer_files_are_rejected() {
        assert!(decode_settings("not ron at all").is_err());
        let newer = decode_settings("(version: 99, settings: ())").unwrap_err();
        assert!(newer.contains("newer version"), "{newer}");
    }

    #[test]
    fn an_unreadable_file_is_kept_as_bak_and_defaults_load() {
        let mut storage = Storage::memory();
        storage.write(SETTINGS_KEY, "{{{").unwrap();
        let (settings, notice) = load(&mut storage, SETTINGS_KEY, "settings", decode_settings);
        assert_eq!(settings, Settings::default());
        assert!(notice.unwrap().contains("settings.bak"));
        assert_eq!(
            storage.read("settings.bak").unwrap().as_deref(),
            Some("{{{")
        );
        // A missing file is simply the first launch: no notice.
        let (_, notice) = load(&mut storage, SAVE_KEY, "progress", decode_progress);
        assert_eq!(notice, None);
    }

    #[test]
    fn data_dir_follows_each_platform() {
        let env = |pairs: &'static [(&'static str, &'static str)]| {
            move |k: &str| {
                pairs
                    .iter()
                    .find(|(key, _)| *key == k)
                    .map(|(_, v)| v.to_string())
            }
        };
        assert_eq!(
            data_dir(
                "windows",
                env(&[("APPDATA", r"C:\Users\a\AppData\Roaming")])
            ),
            Some(PathBuf::from(r"C:\Users\a\AppData\Roaming").join(GAME_DIR))
        );
        assert_eq!(
            data_dir("macos", env(&[("HOME", "/Users/a")])),
            Some(PathBuf::from("/Users/a/Library/Application Support").join(GAME_DIR))
        );
        assert_eq!(
            data_dir("linux", env(&[("HOME", "/home/a")])),
            Some(PathBuf::from("/home/a/.local/share").join(GAME_DIR))
        );
        assert_eq!(
            data_dir(
                "linux",
                env(&[("HOME", "/home/a"), ("XDG_DATA_HOME", "/data")])
            ),
            Some(PathBuf::from("/data").join(GAME_DIR))
        );
        // A relative XDG path is ignored, as the spec says.
        assert_eq!(
            data_dir(
                "linux",
                env(&[("HOME", "/home/a"), ("XDG_DATA_HOME", "rel")])
            ),
            Some(PathBuf::from("/home/a/.local/share").join(GAME_DIR))
        );
        assert_eq!(data_dir("linux", env(&[])), None);
    }

    #[test]
    fn a_folder_store_writes_whole_files() {
        let dir = std::env::temp_dir().join(format!("lib-save-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let mut storage = Storage::in_dir(&dir);
        assert_eq!(storage.read(SAVE_KEY), Ok(None));
        storage.write(SAVE_KEY, "first").unwrap();
        storage.write(SAVE_KEY, "second").unwrap();
        assert_eq!(storage.read(SAVE_KEY), Ok(Some("second".into())));
        assert!(!dir.join("save.ron.tmp").exists());
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
