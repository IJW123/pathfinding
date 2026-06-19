//! In-memory level layout: glam/`Commodity`-typed spec structs the constructors consume, plus the
//! `From<Raw…>` mappings off the serde [`manifest`](crate::objects::manifest) layer. Mirrors
//! `sprites::catalog` (which holds `From<RawSpriteDef>`): the in-memory side imports the raw side,
//! never the reverse.

use bevy::prelude::*;

use logistics::commodity::Commodity;
use world::elevation::config::{FeaturePopulation, TerrainConfig};
use world::elevation::generation::feature::FeatureSpec;

use crate::objects::manifest::{
    ObstacleShape, RawCarrierSpec, RawFeatureSpec, RawLevelSpec, RawObstacleSpec, RawStorageSpec,
    RawTerrainSpec,
};

/// The whole authored level — map size, terrain recipe, and every spawned object — loaded once at
/// plugin build (see [`crate::objects::loader`]) and read by `spawn_level`. A resource so the spawn
/// system takes it as `Res<LevelSpec>`.
#[derive(Resource)]
pub struct LevelSpec {
    /// Half the square map's side; feeds both the boundary walls and [`LevelSpec::terrain_config`].
    pub map_half_extent: f32,
    pub terrain: TerrainSpec,
    pub obstacles: Vec<ObstacleSpec>,
    pub storage: StorageSpec,
    pub carrier: CarrierSpec,
}

/// The procedural terrain recipe, in `world`'s own types. `half_extent` is not stored here — it's
/// the shared [`LevelSpec::map_half_extent`], applied when building the [`TerrainConfig`].
pub struct TerrainSpec {
    pub seed: u32,
    pub hills: FeaturePopulation,
    pub mountains: FeaturePopulation,
    pub authored: Vec<FeatureSpec>,
}

impl LevelSpec {
    /// Assemble the `world`-side [`TerrainConfig`] this level drives, stamping the shared
    /// `map_half_extent` as its `half_extent`. Replaces the old `terrain::level_terrain()`.
    #[must_use]
    pub fn terrain_config(&self) -> TerrainConfig {
        TerrainConfig {
            half_extent: self.map_half_extent,
            seed: self.terrain.seed,
            hills: self.terrain.hills,
            mountains: self.terrain.mountains,
            authored: self.terrain.authored.clone(),
        }
    }
}

/// One interior obstacle: a silhouette `shape` scaled to `size`, placed at `pos`/`rotation`, either
/// static or `pushable`. `shape` is the shared [`ObstacleShape`] (not mirrored).
pub struct ObstacleSpec {
    pub shape: ObstacleShape,
    pub pos: Vec2,
    pub rotation: f32,
    pub size: f32,
    pub pushable: bool,
}

/// The storage building: square of `half_extent`, capped by `max_volume`, with a `dock_radius`
/// load/unload zone and a starting `stock`.
pub struct StorageSpec {
    pub pos: Vec2,
    pub half_extent: f32,
    pub max_volume: f32,
    pub dock_radius: f32,
    pub stock: Vec<(Commodity, u32)>,
}

/// The player/carrier: spawned at `spawn`, capped on both `max_weight` and `max_volume`.
pub struct CarrierSpec {
    pub spawn: Vec2,
    pub max_weight: f32,
    pub max_volume: f32,
}

impl From<RawLevelSpec> for LevelSpec {
    fn from(raw: RawLevelSpec) -> Self {
        Self {
            map_half_extent: raw.map_half_extent,
            terrain: raw.terrain.into(),
            obstacles: raw.obstacles.into_iter().map(ObstacleSpec::from).collect(),
            storage: raw.storage.into(),
            carrier: raw.carrier.into(),
        }
    }
}

impl From<RawTerrainSpec> for TerrainSpec {
    fn from(raw: RawTerrainSpec) -> Self {
        // The `world` fields (FeaturePopulation/FeatureSpec) are built via the raw types' inherent
        // mappers — there's no `From<Raw…>` for them (foreign `Self`, orphan rule), same boundary as
        // `RawCommodity::to_commodity`.
        Self {
            seed: raw.seed,
            hills: raw.hills.to_population(),
            mountains: raw.mountains.to_population(),
            authored: raw
                .authored
                .into_iter()
                .map(RawFeatureSpec::to_feature)
                .collect(),
        }
    }
}

impl From<RawObstacleSpec> for ObstacleSpec {
    fn from(raw: RawObstacleSpec) -> Self {
        Self {
            shape: raw.shape,
            pos: Vec2::new(raw.pos.0, raw.pos.1),
            rotation: raw.rotation,
            size: raw.size,
            pushable: raw.pushable,
        }
    }
}

impl From<RawStorageSpec> for StorageSpec {
    fn from(raw: RawStorageSpec) -> Self {
        Self {
            pos: Vec2::new(raw.pos.0, raw.pos.1),
            half_extent: raw.half_extent,
            max_volume: raw.max_volume,
            dock_radius: raw.dock_radius,
            stock: raw
                .stock
                .into_iter()
                .map(|(commodity, amount)| (commodity.to_commodity(), amount))
                .collect(),
        }
    }
}

impl From<RawCarrierSpec> for CarrierSpec {
    fn from(raw: RawCarrierSpec) -> Self {
        Self {
            spawn: Vec2::new(raw.spawn.0, raw.spawn.1),
            max_weight: raw.max_weight,
            max_volume: raw.max_volume,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ron_into_in_memory_spec() {
        // The round-trip: RON text → raw (serde) → in-memory. Asserts the shape enum survives, the
        // tuple becomes a `Vec2`, and `RawCommodity` maps to the domain `Commodity`.
        let text = r#"(
            map_half_extent: 2000.0,
            terrain: (
                seed: 0x5EED1234,
                hills:     (count: 36, radius_min: 120.0, radius_max: 280.0, height_min: 18.0, height_max: 45.0),
                mountains: (count: 6,  radius_min: 350.0, radius_max: 600.0, height_min: 70.0, height_max: 100.0),
                authored: [(center: (-900.0, 700.0), radius: 500.0, height: 95.0)],
            ),
            obstacles: [
                (shape: Circle, pos: (250.0, 0.0), rotation: 0.0, size: 60.0, pushable: false),
                (shape: Quad, pos: (150.0, -260.0), rotation: 0.5, size: 74.0, pushable: true),
            ],
            storage: (pos: (-250.0, 200.0), half_extent: 50.0, max_volume: 20.0, dock_radius: 120.0,
                      stock: [(Grain, 100), (Coal, 40)]),
            carrier: (spawn: (0.0, 0.0), max_weight: 2000.0, max_volume: 3.0),
        )"#;

        let raw: RawLevelSpec = ron::from_str(text).expect("valid level RON");
        let spec = LevelSpec::from(raw);

        assert_eq!(spec.map_half_extent, 2000.0);
        assert_eq!(spec.terrain.seed, 0x5EED1234);
        assert_eq!(spec.terrain.hills.count, 36);
        assert_eq!(spec.terrain.authored[0].center, Vec2::new(-900.0, 700.0));
        assert_eq!(spec.obstacles.len(), 2);
        assert!(matches!(spec.obstacles[0].shape, ObstacleShape::Circle));
        assert_eq!(spec.obstacles[0].pos, Vec2::new(250.0, 0.0));
        assert!(spec.obstacles[1].pushable);
        assert_eq!(spec.obstacles[1].rotation, 0.5);
        assert_eq!(
            spec.storage.stock,
            vec![(Commodity::Grain, 100), (Commodity::Coal, 40)]
        );
        assert_eq!(spec.carrier.max_weight, 2000.0);

        // The shared map size becomes the terrain's half_extent.
        assert_eq!(spec.terrain_config().half_extent, 2000.0);
    }
}
