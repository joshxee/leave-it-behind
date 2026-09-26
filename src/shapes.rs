//! Placeholder primitives: every visual is a [`Sprite`]. Rectangles use the
//! default white texture; circles and rings use two small textures generated
//! at build time, tinted by `Sprite::color`. Headless apps (no
//! `Assets<Image>`) get default handles, which is fine because nothing renders.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

/// Texture size of the generated disc and ring.
const TEXTURE_SIZE: u32 = 64;
/// Inner radius of the ring texture, as a fraction of the outer radius.
pub const RING_INNER: f32 = 0.7;

#[derive(Resource, Debug, Clone, Default)]
pub struct Shapes {
    pub disc: Handle<Image>,
    pub ring: Handle<Image>,
}

impl Shapes {
    /// A filled circle of `radius` world units.
    pub fn circle(&self, radius: f32, color: Color) -> Sprite {
        Sprite {
            image: self.disc.clone(),
            color,
            custom_size: Some(Vec2::splat(radius * 2.0)),
            ..default()
        }
    }

    /// A ring of outer `radius`; the hole is [`RING_INNER`] of the radius.
    pub fn ring(&self, radius: f32, color: Color) -> Sprite {
        Sprite {
            image: self.ring.clone(),
            color,
            custom_size: Some(Vec2::splat(radius * 2.0)),
            ..default()
        }
    }
}

/// An axis-aligned rectangle sprite.
pub fn rect(size: Vec2, color: Color) -> Sprite {
    Sprite::from_color(color, size)
}

/// Transform for a sprite at `pos` on layer `z`.
pub fn at(pos: Vec2, z: f32) -> Transform {
    Transform::from_translation(pos.extend(z))
}

pub struct ShapesPlugin;

impl Plugin for ShapesPlugin {
    fn build(&self, app: &mut App) {
        let shapes = match app.world_mut().get_resource_mut::<Assets<Image>>() {
            Some(mut images) => Shapes {
                disc: images.add(disc_image(TEXTURE_SIZE, 0.0)),
                ring: images.add(disc_image(TEXTURE_SIZE, RING_INNER)),
            },
            None => Shapes::default(),
        };
        app.insert_resource(shapes);
    }
}

/// White disc (or ring, when `inner > 0`) with a one-pixel antialiased edge.
fn disc_image(size: u32, inner: f32) -> Image {
    let half = size as f32 / 2.0;
    let pixel = 1.0 / half;
    let mut data = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let d = (Vec2::new(x as f32 + 0.5, y as f32 + 0.5) - Vec2::splat(half)).length() / half;
            let alpha = coverage(d, inner, pixel);
            data.extend_from_slice(&[255, 255, 255, (alpha * 255.0).round() as u8]);
        }
    }
    Image::new(
        Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

/// Alpha at normalized distance `d` from the center (1 = outer edge).
fn coverage(d: f32, inner: f32, pixel: f32) -> f32 {
    let outer = ((1.0 - d) / pixel + 0.5).clamp(0.0, 1.0);
    let hole = if inner > 0.0 {
        ((d - inner) / pixel + 0.5).clamp(0.0, 1.0)
    } else {
        1.0
    };
    outer * hole
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disc_is_solid_inside_and_clear_outside() {
        assert_eq!(coverage(0.0, 0.0, 0.03), 1.0);
        assert_eq!(coverage(1.2, 0.0, 0.03), 0.0);
    }

    #[test]
    fn ring_has_a_hole() {
        assert_eq!(coverage(0.2, RING_INNER, 0.03), 0.0);
        assert_eq!(coverage(0.85, RING_INNER, 0.03), 1.0);
    }
}
