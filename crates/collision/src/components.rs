use bevy::prelude::*;

use crate::hull::{ConvexHull, HullError};
use crate::shape::ColliderShape;

#[derive(Component)]
pub struct Collider {
    pub shape: ColliderShape,
}

impl Collider {
    #[must_use]
    pub fn obb(half_extents: Vec2) -> Self {
        Self {
            shape: ColliderShape::Obb { half_extents },
        }
    }

    #[must_use]
    pub fn circle(radius: f32) -> Self {
        Self {
            shape: ColliderShape::Circle { radius },
        }
    }

    /// # Errors
    /// See [`ConvexHull::new`] — fewer than 3 points, concave, or zero-area input.
    pub fn convex(points: Vec<Vec2>) -> Result<Self, HullError> {
        ConvexHull::new(points).map(|hull| Self {
            shape: ColliderShape::Convex { hull },
        })
    }

    /// Full span for sprite sizing. Obb → `half*2`, Circle → `splat(r*2)`, Convex → local-AABB
    /// span. Convex render assumes an origin-centered hull (sprites center on the transform).
    #[must_use]
    pub fn render_size(&self) -> Vec2 {
        match &self.shape {
            ColliderShape::Obb { half_extents } => *half_extents * 2.0,
            ColliderShape::Circle { radius } => Vec2::splat(radius * 2.0),
            ColliderShape::Convex { hull } => {
                let (min, max) = hull.points().iter().fold(
                    (Vec2::splat(f32::INFINITY), Vec2::splat(f32::NEG_INFINITY)),
                    |(min, max), &p| (min.min(p), max.max(p)),
                );
                max - min
            }
        }
    }
}

#[derive(Component)]
pub struct Solid;

#[derive(Component)]
pub struct Static;
