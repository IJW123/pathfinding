use bevy::prelude::*;

use crate::hull::ConvexHull;

/// Local-space collider geometry. Boxes are zero-or-nonzero-rotation OBBs; there is no
/// axis-aligned special case.
pub enum ColliderShape {
    Obb { half_extents: Vec2 },
    Convex { hull: ConvexHull }, // local-space; CCW + convexity guaranteed by the type
    Circle { radius: f32 },
}
