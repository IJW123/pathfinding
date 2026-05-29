use bevy::math::Vec2;
use bevy::prelude::Resource;

use crate::elevation::constants::{
    ELEVATION_CELL, FBM_BASE_FREQ, FBM_GAIN, FBM_LACUNARITY, FBM_OCTAVES, FBM_SEED, HEIGHT_MAX,
    HEIGHT_MIN,
};
use crate::elevation::noise::fbm::fbm;

#[derive(Resource, Clone, Copy)]
pub struct HeightFn {
    pub seed: u32,
    pub octaves: u32,
    pub lacunarity: f32,
    pub gain: f32,
    pub base_freq: f32,
    pub height_min: f32,
    pub height_max: f32,
}

impl Default for HeightFn {
    fn default() -> Self {
        Self {
            seed: FBM_SEED,
            octaves: FBM_OCTAVES,
            lacunarity: FBM_LACUNARITY,
            gain: FBM_GAIN,
            base_freq: FBM_BASE_FREQ,
            height_min: HEIGHT_MIN,
            height_max: HEIGHT_MAX,
        }
    }
}

impl HeightFn {
    #[must_use]
    pub fn sample(&self, pos: Vec2) -> f32 {
        let normalized = fbm(pos, self.seed, self.octaves, self.lacunarity, self.gain, self.base_freq);
        self.height_min + normalized * (self.height_max - self.height_min)
    }

    #[must_use]
    pub fn gradient(&self, pos: Vec2) -> Vec2 {
        let eps = ELEVATION_CELL;
        let dx = self.sample(pos + Vec2::new(eps, 0.0)) - self.sample(pos - Vec2::new(eps, 0.0));
        let dy = self.sample(pos + Vec2::new(0.0, eps)) - self.sample(pos - Vec2::new(0.0, eps));
        Vec2::new(dx, dy) / (2.0 * eps)
    }
}
