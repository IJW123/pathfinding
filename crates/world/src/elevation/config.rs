use bevy::prelude::Resource;

use crate::elevation::generation::feature::FeatureSpec;

/// The terrain recipe: *what* populates this map. Defined here (the engine seam) but authored and
/// inserted by the `level` crate — `world` holds no recipe values of its own. Consumed once at
/// startup to build the immutable [`HeightField`](crate::elevation::height_field::HeightField).
#[derive(Resource, Clone)]
pub struct TerrainConfig {
    /// Half the side length of the square map (full map is `2 * half_extent` per axis, centered on
    /// the origin). Drives the height-field grid, chunk tiling, and feature scatter bounds.
    pub half_extent: f32,
    /// Seed for deterministic procedural feature placement.
    pub seed: u32,
    /// Small, common hills.
    pub hills: FeaturePopulation,
    /// Large, rare mountains.
    pub mountains: FeaturePopulation,
    /// Hand-placed features, always stamped in addition to the procedural ones.
    pub authored: Vec<FeatureSpec>,
}

/// One procedurally-scattered feature class: how many, and the ranges their radius/height are drawn
/// from.
#[derive(Clone, Copy)]
pub struct FeaturePopulation {
    pub count: u32,
    pub radius_min: f32,
    pub radius_max: f32,
    pub height_min: f32,
    pub height_max: f32,
}
