use bevy::prelude::*;
use hitboxes::shape::ColliderShape;

use crate::aabb::Aabb;
use crate::world::{rotation_cos_sin, world_poly};

/// World-space bounds for the broad phase. Circle = ±radius; Obb uses the closed-form
/// rotated-box extent; Convex takes min/max of its rotated, translated vertices (off-center safe).
#[must_use]
pub fn world_aabb(shape: &ColliderShape, transform: &Transform) -> Aabb {
    let center = transform.translation.truncate();
    match shape {
        ColliderShape::Circle { radius } => Aabb {
            min: center - Vec2::splat(*radius),
            max: center + Vec2::splat(*radius),
        },
        ColliderShape::Obb { half_extents } => {
            let cos_sin = rotation_cos_sin(transform);
            let ext = Vec2::new(
                cos_sin.x.abs() * half_extents.x + cos_sin.y.abs() * half_extents.y,
                cos_sin.y.abs() * half_extents.x + cos_sin.x.abs() * half_extents.y,
            );
            Aabb {
                min: center - ext,
                max: center + ext,
            }
        }
        ColliderShape::Convex { hull } => {
            let (min, max) = world_poly(hull.points(), transform).iter().fold(
                (Vec2::splat(f32::INFINITY), Vec2::splat(f32::NEG_INFINITY)),
                |(min, max), &p| (min.min(p), max.max(p)),
            );
            Aabb { min, max }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hitboxes::hull::ConvexHull;
    use std::f32::consts::FRAC_PI_4;

    fn close_vec(a: Vec2, b: Vec2) -> bool {
        (a - b).length() < 1e-3
    }

    #[test]
    fn aabb_axis_aligned_obb() {
        let aabb = world_aabb(
            &ColliderShape::Obb {
                half_extents: Vec2::new(1.0, 2.0),
            },
            &Transform::IDENTITY,
        );
        assert!(close_vec(aabb.min, Vec2::new(-1.0, -2.0)));
        assert!(close_vec(aabb.max, Vec2::new(1.0, 2.0)));
    }

    #[test]
    fn aabb_45deg_square_grows_by_sqrt2() {
        let tx = Transform::from_rotation(Quat::from_rotation_z(FRAC_PI_4));
        let aabb = world_aabb(
            &ColliderShape::Obb {
                half_extents: Vec2::splat(1.0),
            },
            &tx,
        );
        let s = 2.0_f32.sqrt();
        assert!(close_vec(aabb.min, Vec2::splat(-s)));
        assert!(close_vec(aabb.max, Vec2::splat(s)));
    }

    #[test]
    fn aabb_circle_rotation_invariant() {
        let tx = Transform::from_xyz(3.0, 0.0, 0.0).with_rotation(Quat::from_rotation_z(FRAC_PI_4));
        let aabb = world_aabb(&ColliderShape::Circle { radius: 1.0 }, &tx);
        assert!(close_vec(aabb.min, Vec2::new(2.0, -1.0)));
        assert!(close_vec(aabb.max, Vec2::new(4.0, 1.0)));
    }

    #[test]
    fn aabb_off_center_convex() {
        // Hull occupying local x∈[0,2], y∈[0,1] — not origin-centered.
        let hull = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(2.0, 0.0),
            Vec2::new(2.0, 1.0),
            Vec2::new(0.0, 1.0),
        ];
        let hull = ConvexHull::new(hull).expect("valid hull");
        let tx = Transform::from_xyz(10.0, 10.0, 0.0);
        let aabb = world_aabb(&ColliderShape::Convex { hull }, &tx);
        assert!(close_vec(aabb.min, Vec2::new(10.0, 10.0)));
        assert!(close_vec(aabb.max, Vec2::new(12.0, 11.0)));
    }
}
