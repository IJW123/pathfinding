use world::elevation::config::{FeaturePopulation, TerrainConfig};

use crate::constants::{
    AUTHORED_FEATURES, FEATURE_SEED, HILL_COUNT, HILL_HEIGHT_MAX, HILL_HEIGHT_MIN, HILL_RADIUS_MAX,
    HILL_RADIUS_MIN, MAP_HALF_EXTENT, MOUNTAIN_COUNT, MOUNTAIN_HEIGHT_MAX, MOUNTAIN_HEIGHT_MIN,
    MOUNTAIN_RADIUS_MAX, MOUNTAIN_RADIUS_MIN,
};

/// This level's terrain recipe, assembled from the authored constants. Inserted as a resource by
/// [`LevelPlugin`](crate::plugin::LevelPlugin) for `world`'s elevation engine to consume.
#[must_use]
pub fn level_terrain() -> TerrainConfig {
    TerrainConfig {
        half_extent: MAP_HALF_EXTENT,
        seed: FEATURE_SEED,
        hills: FeaturePopulation {
            count: HILL_COUNT,
            radius_min: HILL_RADIUS_MIN,
            radius_max: HILL_RADIUS_MAX,
            height_min: HILL_HEIGHT_MIN,
            height_max: HILL_HEIGHT_MAX,
        },
        mountains: FeaturePopulation {
            count: MOUNTAIN_COUNT,
            radius_min: MOUNTAIN_RADIUS_MIN,
            radius_max: MOUNTAIN_RADIUS_MAX,
            height_min: MOUNTAIN_HEIGHT_MIN,
            height_max: MOUNTAIN_HEIGHT_MAX,
        },
        authored: AUTHORED_FEATURES.to_vec(),
    }
}
