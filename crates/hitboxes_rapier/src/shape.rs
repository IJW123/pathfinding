use std::error::Error;
use std::fmt::{self, Display, Formatter};

use bevy::prelude::*;
use parry2d::shape::{Ball, ConvexPolygon, Cuboid, Shape, SharedShape};

use crate::convert::{parry_to_vec2, vec2_to_parry};

/// Local-space collider geometry over parry shapes. Obb rotation comes from the entity's
/// `Transform`; convex hulls are computed by parry (see [`ColliderShape::convex`]).
pub enum ColliderShape {
    Obb(Cuboid),
    Circle(Ball),
    Convex(ConvexPolygon),
}

/// Input points span no area: fewer than 3 distinct points, or all collinear.
#[derive(Debug, PartialEq, Eq)]
pub struct DegenerateHullError;

impl Display for DegenerateHullError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "convex hull needs at least 3 non-collinear points")
    }
}

impl Error for DegenerateHullError {}

impl ColliderShape {
    /// Convex hull of an arbitrary point cloud, computed by parry. Unlike the old `hitboxes`
    /// crate this accepts concave or unordered input (the hull wraps it) and may merge
    /// duplicate or collinear vertices.
    ///
    /// # Errors
    /// [`DegenerateHullError`] when the points span no area.
    pub fn convex(points: &[Vec2]) -> Result<Self, DegenerateHullError> {
        let points: Vec<_> = points.iter().copied().map(vec2_to_parry).collect();
        ConvexPolygon::from_convex_hull(&points)
            .map(Self::Convex)
            .ok_or(DegenerateHullError)
    }

    #[must_use]
    pub fn to_shared_shape(&self) -> SharedShape {
        match self {
            Self::Obb(cuboid) => SharedShape::new(*cuboid),
            Self::Circle(ball) => SharedShape::new(*ball),
            Self::Convex(polygon) => SharedShape::new(polygon.clone()),
        }
    }

    /// Full local-space span: Obb/Circle closed-form, Convex via its local AABB (off-center
    /// hulls keep their true span).
    #[must_use]
    pub fn local_extent(&self) -> Vec2 {
        match self {
            Self::Obb(cuboid) => parry_to_vec2(cuboid.half_extents) * 2.0,
            Self::Circle(ball) => Vec2::splat(ball.radius * 2.0),
            Self::Convex(polygon) => parry_to_vec2(polygon.compute_local_aabb().extents()),
        }
    }

    /// CCW hull vertices for rendering; `None` for analytic shapes.
    #[must_use]
    pub fn hull_points(&self) -> Option<Vec<Vec2>> {
        match self {
            Self::Convex(polygon) => Some(
                polygon
                    .points()
                    .iter()
                    .copied()
                    .map(parry_to_vec2)
                    .collect(),
            ),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn concave_cloud_is_hulled_not_rejected() {
        // Square corners plus an interior point: hull keeps the 4 corners.
        let shape = ColliderShape::convex(&[
            Vec2::new(0.0, 0.0),
            Vec2::new(2.0, 0.0),
            Vec2::new(2.0, 2.0),
            Vec2::new(0.0, 2.0),
            Vec2::new(1.0, 1.0),
        ])
        .expect("valid hull");
        assert_eq!(shape.hull_points().expect("convex").len(), 4);
    }

    #[test]
    fn collinear_points_rejected() {
        let result = ColliderShape::convex(&[
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(2.0, 2.0),
        ]);
        assert_eq!(result.err(), Some(DegenerateHullError));
    }

    #[test]
    fn too_few_points_rejected() {
        let result = ColliderShape::convex(&[Vec2::ZERO, Vec2::X]);
        assert_eq!(result.err(), Some(DegenerateHullError));
    }

    #[test]
    fn local_extent_per_variant() {
        let obb = ColliderShape::Obb(Cuboid::new(vec2_to_parry(Vec2::new(1.0, 2.0))));
        assert_eq!(obb.local_extent(), Vec2::new(2.0, 4.0));

        let circle = ColliderShape::Circle(Ball::new(3.0));
        assert_eq!(circle.local_extent(), Vec2::splat(6.0));

        // Off-center hull spanning x∈[0,2], y∈[0,1]: span survives, not ±max.
        let convex = ColliderShape::convex(&[
            Vec2::new(0.0, 0.0),
            Vec2::new(2.0, 0.0),
            Vec2::new(2.0, 1.0),
            Vec2::new(0.0, 1.0),
        ])
        .expect("valid hull");
        assert!((convex.local_extent() - Vec2::new(2.0, 1.0)).length() < 1e-4);
    }

    #[test]
    fn shared_shape_keeps_variant() {
        let shape = ColliderShape::Circle(Ball::new(2.5));
        let shared = shape.to_shared_shape();
        assert!((shared.as_ball().expect("ball").radius - 2.5).abs() < f32::EPSILON);
    }
}
