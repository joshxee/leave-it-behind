//! Pixel-art sheets shared by presentation systems. Gameplay never depends on loading.
use std::collections::BTreeMap;

use bevy::prelude::*;

struct Sheet {
    image: Handle<Image>,
    cell: Vec2,
    columns: usize,
    frames: usize,
}

#[derive(Resource, Default)]
pub struct MaintenanceArt(BTreeMap<&'static str, Sheet>);

impl MaintenanceArt {
    pub fn sprite(&self, name: &str, frame: usize, size: Vec2) -> Sprite {
        let sheet = &self.0[name];
        Sprite {
            image: sheet.image.clone(),
            rect: Some(sheet.rect(frame)),
            custom_size: Some(size),
            ..default()
        }
    }

    pub fn frame(&self, sprite: &mut Sprite, name: &str, frame: usize) {
        let sheet = &self.0[name];
        sprite.image = sheet.image.clone();
        sprite.rect = Some(sheet.rect(frame));
    }
}

impl Sheet {
    fn rect(&self, frame: usize) -> Rect {
        let frame = frame.min(self.frames - 1);
        let min =
            Vec2::new((frame % self.columns) as f32, (frame / self.columns) as f32) * self.cell;
        Rect::from_corners(min, min + self.cell)
    }
}

pub struct MaintenanceArtPlugin;

pub const SHEETS: &[(&str, u32, u32, usize)] = &[
    ("radar", 128, 144, 1),
    ("radar-sweep", 104, 104, 24),
    ("nav-ship", 16, 20, 2),
    ("helm-base", 32, 32, 1),
    ("helm-stick", 12, 18, 1),
    ("engine", 224, 88, 4),
    ("engine-vertical", 128, 400, 4),
    ("wall-depth", 64, 88, 256),
    ("door-depth", 64, 88, 5),
    ("bolt", 32, 32, 9),
    ("tape-strip", 32, 12, 6),
    ("breach", 40, 32, 1),
];

impl Plugin for MaintenanceArtPlugin {
    fn build(&self, app: &mut App) {
        let server = app.world().get_resource::<AssetServer>();
        let mut art = MaintenanceArt::default();
        for &(name, w, h, frames) in SHEETS {
            art.0.insert(
                name,
                Sheet {
                    image: server.map_or_else(Handle::default, |s| {
                        s.load_builder()
                            .with_settings(|s: &mut bevy::image::ImageLoaderSettings| {
                                s.sampler = bevy::image::ImageSampler::nearest()
                            })
                            .load(format!("maintenance/{name}.png"))
                    }),
                    cell: Vec2::new(w as f32, h as f32),
                    columns: frames.min(32),
                    frames,
                },
            );
        }
        app.insert_resource(art);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_cells_match_the_exported_manifest() {
        let manifest: serde_json::Value =
            serde_json::from_str(include_str!("../../assets/maintenance/manifest.json")).unwrap();
        for &(name, width, height, frames) in SHEETS {
            let entry = manifest["assets"]
                .as_array()
                .unwrap()
                .iter()
                .find(|a| a["id"] == name)
                .unwrap();
            assert_eq!(
                entry["frame_size"],
                serde_json::json!([width, height]),
                "{name}"
            );
            assert_eq!(entry["frames"], frames, "{name}");
            assert_eq!(entry["columns"], frames.min(32), "{name}");
        }
    }
}
