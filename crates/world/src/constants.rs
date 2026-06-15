/// Half the side length of the fixed, square playable map (full map is
/// `2 * MAP_HALF_EXTENT` per axis, centered on the origin). Single source of
/// truth for boundary walls and terrain streaming extent.
pub const MAP_HALF_EXTENT: f32 = 2000.0;
