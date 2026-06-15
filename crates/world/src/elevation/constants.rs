pub const ELEVATION_CELL: f32 = 10.0;
pub const ELEV_CHUNK_CELLS: usize = 32;

pub const HEIGHT_MIN: f32 = 0.0;
/// Upper clamp on stored heights, and the ceiling `ContourLevels` derives from. Set well
/// above the tallest single feature (~mountain height x detail overshoot) so legitimate
/// peaks aren't flat-topped; upper levels no terrain reaches simply emit no contours.
pub const HEIGHT_MAX: f32 = 250.0;

pub const FBM_SEED: u32 = 0xC0FFEE;
pub const FBM_OCTAVES: u32 = 4;
pub const FBM_LACUNARITY: f32 = 2.0;
pub const FBM_GAIN: f32 = 0.5;
pub const FBM_BASE_FREQ: f32 = 1.0 / 200.0;

// --- Heightmap generation: flat base ---
/// Amplitude of the gentle undulation on otherwise-flat ground (world height units).
pub const FLAT_AMP: f32 = 4.0;
/// Spatial frequency of the flat-base undulation (low = broad, slow rolls).
pub const FLAT_FREQ: f32 = 1.0 / 600.0;

// --- Heightmap generation: per-feature detail (shape, not placement) ---
/// Fractional noise applied to each feature's falloff to break perfect-circle domes.
pub const HILL_ROUGHNESS: f32 = 0.35;
/// Spatial frequency of the per-feature detail noise.
pub const HILL_DETAIL_FREQ: f32 = 1.0 / 80.0;
