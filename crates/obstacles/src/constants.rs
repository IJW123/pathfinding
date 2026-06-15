use bevy::math::Vec2;

/// Z layer for obstacles: above contour lines (0.1), below the player (1.0). Render-ordering
/// policy intrinsic to the kind, so it stays here; instance positions live in the `level` crate.
pub const OBSTACLE_Z: f32 = 0.5;

/// Boundary-wall thickness (full span across the wall).
pub const WALL_THICKNESS: f32 = 20.0;

/// Z layer for boundary walls: ground level, below obstacles (0.5) and the player (1.0).
pub const WALL_Z: f32 = 0.0;

/// Static circle radius.
pub const CIRCLE_RADIUS: f32 = 60.0;

/// Static triangle hull. Local-space, CCW, origin-centered (winding/convexity enforced by the
/// hull builder at spawn — CCW here is for readability).
pub const TRIANGLE_POINTS: [Vec2; 3] = [
    Vec2::new(-60.0, -45.0),
    Vec2::new(70.0, -25.0),
    Vec2::new(-10.0, 70.0),
];

/// Pushable irregular convex quad. Local-space, CCW, origin-centered.
pub const QUAD_POINTS: [Vec2; 4] = [
    Vec2::new(-55.0, -40.0),
    Vec2::new(50.0, -55.0),
    Vec2::new(65.0, 35.0),
    Vec2::new(-40.0, 50.0),
];

/// Pushable pentagon. Local-space, CCW, origin-centered.
pub const PENTAGON_POINTS: [Vec2; 5] = [
    Vec2::new(0.0, 65.0),
    Vec2::new(-62.0, 20.0),
    Vec2::new(-38.0, -53.0),
    Vec2::new(38.0, -53.0),
    Vec2::new(62.0, 20.0),
];
