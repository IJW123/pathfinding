pub const CAMERA_PAN_SPEED: f32 = 500.0;

/// Metres visible top-to-bottom at zoom `scale = 1.0`. With
/// `ScalingMode::FixedVertical` this stays constant across window resizes.
pub const DEFAULT_VIEW_HEIGHT_M: f32 = 1000.0;

/// Per-second exponential zoom factor: `scale *= ZOOM_SPEED.powf(dt)` zooms out,
/// the reciprocal zooms in. `2.5` ≈ visible extent ×2.5 (or ÷2.5) per second held.
pub const ZOOM_SPEED: f32 = 2.5;

/// Smallest projection scale (most zoomed in): ~100 m visible vertically.
pub const ZOOM_MIN: f32 = 0.1;
/// Largest projection scale (most zoomed out): ~5 km visible, covers the 4 km map.
pub const ZOOM_MAX: f32 = 5.0;
