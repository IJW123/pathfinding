use bevy::prelude::*;
use parry2d::shape::{Ball, Cuboid};

use crate::convert::vec2_to_parry;
use crate::shape::{ColliderShape, DegenerateHullError};

#[derive(Component)]
pub struct Collider {
    pub shape: ColliderShape,
}

impl Collider {
    #[must_use]
    pub fn obb(half_extents: Vec2) -> Self {
        Self {
            shape: ColliderShape::Obb(Cuboid::new(vec2_to_parry(half_extents))),
        }
    }

    #[must_use]
    pub fn circle(radius: f32) -> Self {
        Self {
            shape: ColliderShape::Circle(Ball::new(radius)),
        }
    }

    /// Convex hull of `points`, computed by parry — concave clouds are wrapped, not rejected.
    ///
    /// # Errors
    /// See [`ColliderShape::convex`] — degenerate (zero-area) input.
    pub fn convex(points: Vec<Vec2>) -> Result<Self, DegenerateHullError> {
        ColliderShape::convex(&points).map(|shape| Self { shape })
    }

    /// Full span for sprite sizing.
    #[must_use]
    pub fn render_size(&self) -> Vec2 {
        self.shape.local_extent()
    }
}

#[derive(Component)]
pub struct Solid;

#[derive(Component)]
pub struct Static;
