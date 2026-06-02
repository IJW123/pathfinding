use bevy::math::{FloatExt, Vec2};

use crate::constants::MAP_HALF_EXTENT;
use crate::elevation::constants::{
    FEATURE_SEED, HILL_COUNT, HILL_HEIGHT_MAX, HILL_HEIGHT_MIN, HILL_RADIUS_MAX, HILL_RADIUS_MIN,
    MOUNTAIN_COUNT, MOUNTAIN_HEIGHT_MAX, MOUNTAIN_HEIGHT_MIN, MOUNTAIN_RADIUS_MAX,
    MOUNTAIN_RADIUS_MIN,
};
use crate::elevation::generation::feature::FeatureSpec;
use crate::elevation::noise::hash::hash_to_unit;

/// Hand-placed features, always stamped in addition to the seeded-random ones.
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

/// All elevation features: authored, then seeded-random hills, then seeded-random mountains.
/// Features may overlap freely; overlapping contributions add and clamp at `HEIGHT_MAX`.
#[must_use]
pub fn all_features() -> Vec<FeatureSpec> {
    let mut features = AUTHORED_FEATURES.to_vec();
    features.extend(random_features(
        FEATURE_SEED,
        HILL_COUNT,
        (HILL_RADIUS_MIN, HILL_RADIUS_MAX),
        (HILL_HEIGHT_MIN, HILL_HEIGHT_MAX),
    ));
    features.extend(random_features(
        FEATURE_SEED ^ 0xA5A5_A5A5,
        MOUNTAIN_COUNT,
        (MOUNTAIN_RADIUS_MIN, MOUNTAIN_RADIUS_MAX),
        (MOUNTAIN_HEIGHT_MIN, MOUNTAIN_HEIGHT_MAX),
    ));
    features
}

/// Deterministically scatter `count` features across the map. Each feature's position,
/// radius, and height come from independent hash streams keyed on its index, so the layout
/// is reproducible and changing `seed` reshuffles it.
#[must_use]
fn random_features(
    seed: u32,
    count: u32,
    radius_range: (f32, f32),
    height_range: (f32, f32),
) -> Vec<FeatureSpec> {
    (0..count as i32)
        .map(|k| {
            let cx = (hash_to_unit(k, 0, seed) * 2.0 - 1.0) * MAP_HALF_EXTENT;
            let cy = (hash_to_unit(k, 1, seed) * 2.0 - 1.0) * MAP_HALF_EXTENT;
            FeatureSpec {
                center: Vec2::new(cx, cy),
                radius: radius_range.0.lerp(radius_range.1, hash_to_unit(k, 2, seed)),
                height: height_range.0.lerp(height_range.1, hash_to_unit(k, 3, seed)),
            }
        })
        .collect()
}
