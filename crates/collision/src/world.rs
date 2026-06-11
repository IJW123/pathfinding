use bevy::prelude::*;
use smallvec::SmallVec;

use crate::shape::ColliderShape;

/// World-space geometry for the narrow phase. `Poly` stays alloc-free up to 8 verts (boxes are 4,
/// typical hulls fit); only larger hulls spill to the heap.
pub enum WorldShape {
    Poly(SmallVec<[Vec2; 8]>),
    Circle(Vec2, f32),
}

/// `(cos, sin)` of the transform's z-rotation, taken from `rot * X` to dodge euler edge cases.
#[must_use]
pub fn rotation_cos_sin(transform: &Transform) -> Vec2 {
    (transform.rotation * Vec3::X).truncate()
}

/// Rotate a local point by `(cos, sin)`.
fn rotate(p: Vec2, cos_sin: Vec2) -> Vec2 {
    Vec2::new(
        p.x * cos_sin.x - p.y * cos_sin.y,
        p.x * cos_sin.y + p.y * cos_sin.x,
    )
}

/// Transform local-space polygon vertices to world space: rotate then translate.
#[must_use]
pub fn world_poly(local: &[Vec2], transform: &Transform) -> SmallVec<[Vec2; 8]> {
    let cos_sin = rotation_cos_sin(transform);
    let offset = transform.translation.truncate();
    local.iter().map(|&p| rotate(p, cos_sin) + offset).collect()
}

/// Lower a collider shape + transform into world-space geometry. Circles are rotation-invariant.
#[must_use]
pub fn to_world(shape: &ColliderShape, transform: &Transform) -> WorldShape {
    match shape {
        ColliderShape::Obb { half_extents } => {
            let h = *half_extents;
            let corners = [
                Vec2::new(-h.x, -h.y),
                Vec2::new(h.x, -h.y),
                Vec2::new(h.x, h.y),
                Vec2::new(-h.x, h.y),
            ];
            WorldShape::Poly(world_poly(&corners, transform))
        }
        ColliderShape::Convex { hull } => WorldShape::Poly(world_poly(hull.points(), transform)),
        ColliderShape::Circle { radius } => {
            WorldShape::Circle(transform.translation.truncate(), *radius)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::FRAC_PI_2;

    fn close_vec(a: Vec2, b: Vec2) -> bool {
        (a - b).length() < 1e-4
    }

    #[test]
    fn world_poly_rotates_then_translates() {
        let tx =
            Transform::from_xyz(10.0, 5.0, 0.0).with_rotation(Quat::from_rotation_z(FRAC_PI_2));
        let verts = world_poly(&[Vec2::new(1.0, 0.0)], &tx);
        // (1,0) rotated 90° → (0,1), then + (10,5).
        assert!(close_vec(verts[0], Vec2::new(10.0, 6.0)), "{:?}", verts[0]);
    }

    #[test]
    fn obb_to_world_emits_four_world_corners() {
        let tx = Transform::from_xyz(2.0, 0.0, 0.0);
        match to_world(
            &ColliderShape::Obb {
                half_extents: Vec2::splat(1.0),
            },
            &tx,
        ) {
            WorldShape::Poly(p) => {
                assert_eq!(p.len(), 4);
                assert!(close_vec(p[0], Vec2::new(1.0, -1.0)));
                assert!(close_vec(p[2], Vec2::new(3.0, 1.0)));
            }
            WorldShape::Circle(..) => panic!("expected poly"),
        }
    }
}
