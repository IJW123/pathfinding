//! The one place sprite scaling lives, so the collider and the texture can never disagree on size.
//! Both consume the same `world_size` (world units for the sprite's longest side) and the same
//! normalized def.

use bevy::prelude::*;

use hitboxes_rapier::components::Collider;
use hitboxes_rapier::shape::DegenerateHullError;

use crate::catalog::SpriteDef;

/// Convex collider from a sprite's normalized hull, scaled to `world_size`. Fallible: the hull is
/// external data and could be degenerate even though the bake tool guards against it.
///
/// # Errors
/// [`DegenerateHullError`] if the scaled hull spans no area.
pub fn collider_for(def: &SpriteDef, world_size: f32) -> Result<Collider, DegenerateHullError> {
    let points: Vec<Vec2> = def.hull.iter().map(|&p| p * world_size).collect();
    Collider::convex(points)
}

/// `Sprite::custom_size` for a sprite scaled to `world_size`. The longest side is `world_size`; the
/// shorter side follows the pixel aspect. Computed from `aspect` alone (the raw pixel dims aren't in
/// the manifest), which is algebraically identical to `(img_w, img_h) * world_size / longest_side`.
#[must_use]
pub fn sprite_size(def: &SpriteDef, world_size: f32) -> Vec2 {
    if def.aspect >= 1.0 {
        Vec2::new(world_size, world_size / def.aspect)
    } else {
        Vec2::new(world_size * def.aspect, world_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_square_def(aspect: f32) -> SpriteDef {
        SpriteDef {
            image_path: String::new(),
            aspect,
            hull: vec![
                Vec2::new(-0.5, -0.5),
                Vec2::new(0.5, -0.5),
                Vec2::new(0.5, 0.5),
                Vec2::new(-0.5, 0.5),
            ],
        }
    }

    #[test]
    fn landscape_fits_longest_side_to_world_size() {
        let size = sprite_size(&unit_square_def(2.0), 100.0);
        assert_eq!(size, Vec2::new(100.0, 50.0));
    }

    #[test]
    fn portrait_fits_longest_side_to_world_size() {
        let size = sprite_size(&unit_square_def(0.5), 100.0);
        assert_eq!(size, Vec2::new(50.0, 100.0));
    }

    #[test]
    fn square_is_uniform() {
        let size = sprite_size(&unit_square_def(1.0), 80.0);
        assert_eq!(size, Vec2::splat(80.0));
    }

    #[test]
    fn collider_scales_hull_to_world_size() {
        let collider = collider_for(&unit_square_def(1.0), 100.0).expect("valid hull");
        // Unit square (side 1.0) scaled by 100 -> 100x100 extent.
        assert!((collider.render_size() - Vec2::splat(100.0)).length() < 1e-3);
    }
}
