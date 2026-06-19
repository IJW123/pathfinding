use bevy::math::Vec2;

use world::elevation::generation::feature::FeatureSpec;

/// Half the side length of this level's square map (full map is `2 * MAP_HALF_EXTENT` per axis,
/// centered on the origin). Single source of truth: feeds both the boundary walls and the terrain
/// recipe.
pub const MAP_HALF_EXTENT: f32 = 2000.0;

/// Seed for this level's deterministic procedural feature placement.
pub const FEATURE_SEED: u32 = 0x5EED_1234;

// --- Interior obstacle sizes (circumradius, world units) ---
// Size is a level-side knob: the obstacle crate owns the silhouette, level decides how big each
// instance is. Change these to scale an obstacle; change the `Transform` in `spawn` to move it.
pub const CIRCLE_RADIUS: f32 = 60.0;
pub const TRIANGLE_SIZE: f32 = 75.0;
pub const QUAD_SIZE: f32 = 74.0;
pub const PENTAGON_SIZE: f32 = 65.0;

/// Storage building half-extent (square, world units). Like the obstacle sizes above, size is the
/// level-side knob — the `infrastructure` crate owns the square silhouette, level picks how big.
/// (Capacity/docking magnitudes live in `objects::constants`, with the composition that uses them.)
pub const STORAGE_HALF_EXTENT: f32 = 50.0;

// --- Procedural feature populations (count + radius/height ranges) ---
/// Small, common hills.
pub const HILL_COUNT: u32 = 36;
pub const HILL_RADIUS_MIN: f32 = 120.0;
pub const HILL_RADIUS_MAX: f32 = 280.0;
pub const HILL_HEIGHT_MIN: f32 = 18.0;
pub const HILL_HEIGHT_MAX: f32 = 45.0;

/// Large, rare mountains.
pub const MOUNTAIN_COUNT: u32 = 6;
pub const MOUNTAIN_RADIUS_MIN: f32 = 350.0;
pub const MOUNTAIN_RADIUS_MAX: f32 = 600.0;
pub const MOUNTAIN_HEIGHT_MIN: f32 = 70.0;
pub const MOUNTAIN_HEIGHT_MAX: f32 = 100.0;

/// Hand-placed elevation features, stamped in addition to the procedural ones.
/// Edit this to put a hill/mountain at a specific spot.
pub const AUTHORED_FEATURES: &[FeatureSpec] = &[
    FeatureSpec {
        center: Vec2::new(0.0, 0.0),
        radius: 220.0,
        height: 40.0,
    },
    FeatureSpec {
        center: Vec2::new(-900.0, 700.0),
        radius: 500.0,
        height: 95.0,
    },
];
