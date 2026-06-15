/// Z layer for obstacles: above contour lines (0.1), below the player (1.0). Render-ordering
/// policy intrinsic to the kind, so it stays here; instance positions live in the `level` crate.
pub const OBSTACLE_Z: f32 = 0.5;

/// Boundary-wall thickness (full span across the wall).
pub const WALL_THICKNESS: f32 = 20.0;

/// Z layer for boundary walls: ground level, below obstacles (0.5) and the player (1.0).
pub const WALL_Z: f32 = 0.0;
