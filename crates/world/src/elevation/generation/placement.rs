use bevy::math::{FloatExt, Vec2};

use crate::elevation::config::{FeaturePopulation, TerrainConfig};
use crate::elevation::generation::feature::FeatureSpec;
use crate::elevation::noise::hash::hash_to_unit;

/// Distinct hash stream for mountains, so they don't land on top of the hills drawn from the same
/// seed. Engine detail (how a second stream is derived), not part of the authored recipe.
const MOUNTAIN_STREAM_MASK: u32 = 0xA5A5_A5A5;

/// All elevation features for a recipe: authored, then procedural hills, then procedural mountains.
/// Features may overlap freely; overlapping contributions add and clamp at `HEIGHT_MAX`.
#[must_use]
pub fn all_features(config: &TerrainConfig) -> Vec<FeatureSpec> {
    let mut features = config.authored.clone();
    features.extend(random_features(
        config.seed,
        &config.hills,
        config.half_extent,
    ));
    features.extend(random_features(
        config.seed ^ MOUNTAIN_STREAM_MASK,
        &config.mountains,
        config.half_extent,
    ));
    features
}

/// Deterministically scatter `pop.count` features across the map. Each feature's position,
/// radius, and height come from independent hash streams keyed on its index, so the layout
/// is reproducible and changing `seed` reshuffles it.
#[must_use]
fn random_features(seed: u32, pop: &FeaturePopulation, half_extent: f32) -> Vec<FeatureSpec> {
    (0..pop.count as i32)
        .map(|k| {
            let cx = (hash_to_unit(k, 0, seed) * 2.0 - 1.0) * half_extent;
            let cy = (hash_to_unit(k, 1, seed) * 2.0 - 1.0) * half_extent;
            FeatureSpec {
                center: Vec2::new(cx, cy),
                radius: pop
                    .radius_min
                    .lerp(pop.radius_max, hash_to_unit(k, 2, seed)),
                height: pop
                    .height_min
                    .lerp(pop.height_max, hash_to_unit(k, 3, seed)),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const HALF_EXTENT: f32 = 2000.0;

    const POP: FeaturePopulation = FeaturePopulation {
        count: 10,
        radius_min: 120.0,
        radius_max: 280.0,
        height_min: 18.0,
        height_max: 45.0,
    };

    fn same(a: &FeatureSpec, b: &FeatureSpec) -> bool {
        a.center == b.center && a.radius == b.radius && a.height == b.height
    }

    #[test]
    fn random_features_is_deterministic() {
        let a = random_features(0xABCD, &POP, HALF_EXTENT);
        let b = random_features(0xABCD, &POP, HALF_EXTENT);
        assert_eq!(a.len(), b.len());
        assert!(a.iter().zip(&b).all(|(x, y)| same(x, y)));
    }

    #[test]
    fn random_features_respects_count_and_bounds() {
        let pop = FeaturePopulation { count: 25, ..POP };
        let features = random_features(0x1234, &pop, HALF_EXTENT);
        assert_eq!(features.len() as u32, pop.count);
        for f in &features {
            assert!(f.center.x.abs() <= HALF_EXTENT);
            assert!(f.center.y.abs() <= HALF_EXTENT);
            assert!((POP.radius_min..=POP.radius_max).contains(&f.radius));
            assert!((POP.height_min..=POP.height_max).contains(&f.height));
        }
    }

    #[test]
    fn different_seed_reshuffles_layout() {
        let a = random_features(1, &POP, HALF_EXTENT);
        let b = random_features(2, &POP, HALF_EXTENT);
        assert!(a.iter().zip(&b).any(|(x, y)| !same(x, y)));
    }

    #[test]
    fn all_features_counts_authored_plus_random() {
        let config = TerrainConfig {
            half_extent: HALF_EXTENT,
            seed: 0xABCD,
            hills: FeaturePopulation { count: 12, ..POP },
            mountains: FeaturePopulation { count: 3, ..POP },
            authored: vec![FeatureSpec {
                center: Vec2::ZERO,
                radius: 200.0,
                height: 40.0,
            }],
        };
        let expected =
            config.authored.len() + config.hills.count as usize + config.mountains.count as usize;
        assert_eq!(all_features(&config).len(), expected);
    }
}
