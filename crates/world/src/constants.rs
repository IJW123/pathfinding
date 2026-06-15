pub const WALL_THICKNESS: f32 = 20.0;

/// Z layer for boundary walls: ground level, below obstacles (0.5) and the player (1.0).
pub const WALL_Z: f32 = 0.0;

/// Half the side length of the fixed, square playable map (full map is
/// `2 * MAP_HALF_EXTENT` per axis, centered on the origin). Single source of
/// truth for boundary walls and terrain streaming extent.
pub const MAP_HALF_EXTENT: f32 = 2000.0;
