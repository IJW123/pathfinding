//! Raw serde-facing mirror of the on-disk `level.ron` format. Positions are plain `(f32, f32)`
//! tuples so no glam `serde` feature is needed; `RawCommodity` mirrors `logistics::Commodity` so
//! that domain crate stays serde-free (same reason `sprites::manifest` mirrors its on-disk types).
//!
//! This layer imports nothing from the in-memory [`crate::objects::spec`] layer — `spec` owns the
//! `From<Raw…>` mappings, keeping the dependency one-way (raw → nothing).

use bevy::math::Vec2;
use serde::Deserialize;

use logistics::commodity::Commodity;
use world::elevation::config::FeaturePopulation;
use world::elevation::generation::feature::FeatureSpec;

/// Top-level authored layout, as written in `level.ron`.
#[derive(Deserialize)]
pub struct RawLevelSpec {
    /// Half the square map's side; shared source of truth for the boundary walls and the terrain.
    pub map_half_extent: f32,
    pub terrain: RawTerrainSpec,
    pub obstacles: Vec<RawObstacleSpec>,
    pub storage: RawStorageSpec,
    pub carrier: RawCarrierSpec,
}

/// The procedural terrain recipe. Mirrors `world`'s `TerrainConfig` minus `half_extent` (that comes
/// from the shared `map_half_extent`), keeping `world` serde-free like [`RawCommodity`] does for
/// `logistics`.
#[derive(Deserialize)]
pub struct RawTerrainSpec {
    pub seed: u32,
    pub hills: RawFeaturePopulation,
    pub mountains: RawFeaturePopulation,
    pub authored: Vec<RawFeatureSpec>,
}

/// Wire mirror of `world`'s [`FeaturePopulation`].
#[derive(Deserialize)]
pub struct RawFeaturePopulation {
    pub count: u32,
    pub radius_min: f32,
    pub radius_max: f32,
    pub height_min: f32,
    pub height_max: f32,
}

impl RawFeaturePopulation {
    /// Map to the domain [`FeaturePopulation`]. Inherent method, not `From` — `FeaturePopulation` is
    /// foreign to `level` (the `Self` type), so the orphan rule forbids the impl (same as
    /// [`RawCommodity::to_commodity`]).
    #[must_use]
    pub fn to_population(self) -> FeaturePopulation {
        FeaturePopulation {
            count: self.count,
            radius_min: self.radius_min,
            radius_max: self.radius_max,
            height_min: self.height_min,
            height_max: self.height_max,
        }
    }
}

/// Wire mirror of `world`'s [`FeatureSpec`] (tuple center → `Vec2`).
#[derive(Deserialize)]
pub struct RawFeatureSpec {
    pub center: (f32, f32),
    pub radius: f32,
    pub height: f32,
}

impl RawFeatureSpec {
    /// Map to the domain [`FeatureSpec`]. Inherent method for the same orphan-rule reason as
    /// [`RawFeaturePopulation::to_population`].
    #[must_use]
    pub fn to_feature(self) -> FeatureSpec {
        FeatureSpec {
            center: Vec2::new(self.center.0, self.center.1),
            radius: self.radius,
            height: self.height,
        }
    }
}

/// Obstacle silhouette kind. Shared *verbatim* with the in-memory `ObstacleSpec` (a level-local
/// fieldless enum — no mirror needed, unlike the foreign `Commodity`). It lives in the raw layer so
/// `spec` can re-use it without the raw layer ever depending on `spec`.
#[derive(Deserialize, Clone, Copy)]
pub enum ObstacleShape {
    Circle,
    Triangle,
    Quad,
    Pentagon,
}

#[derive(Deserialize)]
pub struct RawObstacleSpec {
    pub shape: ObstacleShape,
    pub pos: (f32, f32),
    pub rotation: f32,
    pub size: f32,
    pub pushable: bool,
}

#[derive(Deserialize)]
pub struct RawStorageSpec {
    pub pos: (f32, f32),
    pub half_extent: f32,
    pub max_volume: f32,
    pub dock_radius: f32,
    pub stock: Vec<(RawCommodity, u32)>,
}

#[derive(Deserialize)]
pub struct RawCarrierSpec {
    pub spawn: (f32, f32),
    pub max_weight: f32,
    pub max_volume: f32,
}

/// Wire-format mirror of [`Commodity`] (keeps `logistics` serde-free). Map to the domain enum via
/// [`RawCommodity::to_commodity`].
#[derive(Deserialize, Clone, Copy)]
pub enum RawCommodity {
    Grain,
    Coal,
    Lumber,
    IronOre,
}

impl RawCommodity {
    /// Map this wire variant to the domain [`Commodity`]. Implemented as an inherent method rather
    /// than `From<RawCommodity> for Commodity` because `Commodity` is foreign to `level` and is the
    /// `Self` type — the orphan rule forbids that impl.
    #[must_use]
    pub fn to_commodity(self) -> Commodity {
        match self {
            Self::Grain => Commodity::Grain,
            Self::Coal => Commodity::Coal,
            Self::Lumber => Commodity::Lumber,
            Self::IronOre => Commodity::IronOre,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_commodity_covers_every_domain_variant() {
        // Adding a good to `Commodity` without a matching `RawCommodity` arm must fail here, so the
        // wire enum can't silently drift from the domain enum.
        let mapped = [
            RawCommodity::Grain,
            RawCommodity::Coal,
            RawCommodity::Lumber,
            RawCommodity::IronOre,
        ]
        .map(RawCommodity::to_commodity);

        assert_eq!(
            mapped.len(),
            Commodity::COUNT,
            "raw/domain variant count drift"
        );
        for commodity in Commodity::ALL {
            assert!(
                mapped.contains(&commodity),
                "{commodity:?} unreachable from any RawCommodity"
            );
        }
    }
}
