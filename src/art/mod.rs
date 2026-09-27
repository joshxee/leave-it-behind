//! The prototype art packs (`assets/characters/engineer`,
//! `assets/environment/derelict-ship`): atlas images, their layouts, and the
//! manifest data the game needs. Both atlases load with nearest sampling so
//! pixels stay crisp. Headless apps (no `AssetServer`) get default handles,
//! which is fine because nothing renders.

pub mod engineer;
pub mod maintenance;
pub mod tiles;

use bevy::image::{ImageLoaderSettings, ImageSampler};
use bevy::prelude::*;

pub use tiles::Tile;

/// Runtime asset paths (relative to `assets/`). Builds ship only the files
/// listed in `assets/RUNTIME_ASSETS`, which a unit test keeps in step.
pub const SHIP_ATLAS: &str = "environment/derelict-ship/ship-atlas.png";
pub const ENGINEER_ATLAS: &str = "characters/engineer/engineer-atlas.png";

/// Side of one ship tile, in pixels and world units (art is drawn 1:1).
pub const TILE: f32 = 64.0;

#[derive(Resource, Debug, Clone, Default)]
pub struct Art {
    pub ship: Handle<Image>,
    pub ship_layout: Handle<TextureAtlasLayout>,
    pub engineer: Handle<Image>,
    pub engineer_layout: Handle<TextureAtlasLayout>,
}

impl Art {
    /// A whole ship tile.
    pub fn tile(&self, tile: Tile) -> Sprite {
        Sprite::from_atlas_image(
            self.ship.clone(),
            TextureAtlas {
                layout: self.ship_layout.clone(),
                index: tile.index(),
            },
        )
    }

    /// Part of a ship tile: `rect` in tile-local pixels (y down).
    pub fn tile_part(&self, tile: Tile, rect: Rect) -> Sprite {
        Sprite {
            rect: Some(rect),
            ..self.tile(tile)
        }
    }

    /// An engineer frame (anchor it with [`engineer::ANCHOR`]).
    pub fn engineer(&self, index: usize) -> Sprite {
        Sprite::from_atlas_image(
            self.engineer.clone(),
            TextureAtlas {
                layout: self.engineer_layout.clone(),
                index,
            },
        )
    }
}

pub struct ArtPlugin;

impl Plugin for ArtPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(maintenance::MaintenanceArtPlugin);
        let world = app.world_mut();
        let (Some(server), true) = (
            world.get_resource::<AssetServer>().cloned(),
            world.contains_resource::<Assets<TextureAtlasLayout>>(),
        ) else {
            app.init_resource::<Art>();
            return;
        };
        let pixel_art = |s: &mut ImageLoaderSettings| s.sampler = ImageSampler::nearest();
        let mut layouts = world.resource_mut::<Assets<TextureAtlasLayout>>();
        let art = Art {
            ship: server
                .load_builder()
                .with_settings(pixel_art)
                .load(SHIP_ATLAS),
            ship_layout: layouts.add(TextureAtlasLayout::from_grid(
                UVec2::splat(TILE as u32),
                8,
                8,
                None,
                None,
            )),
            engineer: server
                .load_builder()
                .with_settings(pixel_art)
                .load(ENGINEER_ATLAS),
            engineer_layout: layouts.add(TextureAtlasLayout::from_grid(
                UVec2::splat(engineer::CELL),
                engineer::COLUMNS,
                engineer::ROWS,
                None,
                None,
            )),
        };
        app.insert_resource(art);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_asset_list_covers_every_loaded_file() {
        let listed: Vec<&str> = include_str!("../../assets/RUNTIME_ASSETS")
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .collect();
        for path in [SHIP_ATLAS, ENGINEER_ATLAS, crate::ui::FONT_PATH] {
            assert!(
                listed.contains(&path),
                "{path} missing from assets/RUNTIME_ASSETS"
            );
        }
        for (name, ..) in maintenance::SHEETS {
            let path = format!("maintenance/{name}.png");
            assert!(
                listed.contains(&path.as_str()),
                "{path} missing from runtime assets"
            );
        }
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
        for path in listed {
            assert!(root.join(path).is_file(), "assets/{path} does not exist");
        }
    }
}
