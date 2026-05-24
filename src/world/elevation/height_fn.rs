use bevy::math::Vec2;
use bevy::prelude::Resource;

use crate::constants::{
    ELEVATION_CELL, FBM_BASE_FREQ, FBM_GAIN, FBM_LACUNARITY, FBM_OCTAVES, FBM_SEED, HEIGHT_MAX,
    HEIGHT_MIN,
};

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

fn hash2(ix: i32, iy: i32, seed: u32) -> u32 {
    let mut h = seed
        ^ (ix as u32).wrapping_mul(0x9E37_79B1)
        ^ (iy as u32).wrapping_mul(0x85EB_CA77);
    h ^= h >> 16;
    h = h.wrapping_mul(0x7FEB_352D);
    h ^= h >> 15;
    h = h.wrapping_mul(0x846C_A68B);
    h ^= h >> 16;
    h
}

fn hash_unit(ix: i32, iy: i32, seed: u32) -> f32 {
    (hash2(ix, iy, seed) as f32) / (u32::MAX as f32)
}

fn smoothstep(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

fn value_noise(p: Vec2, seed: u32) -> f32 {
    let ix = p.x.floor() as i32;
    let iy = p.y.floor() as i32;
    let fx = p.x - ix as f32;
    let fy = p.y - iy as f32;

    let v00 = hash_unit(ix, iy, seed);
    let v10 = hash_unit(ix + 1, iy, seed);
    let v01 = hash_unit(ix, iy + 1, seed);
    let v11 = hash_unit(ix + 1, iy + 1, seed);

    let ux = smoothstep(fx);
    let uy = smoothstep(fy);

    let a = v00 + (v10 - v00) * ux;
    let b = v01 + (v11 - v01) * ux;
    a + (b - a) * uy
}

fn fbm(pos: Vec2, seed: u32, octaves: u32, lacunarity: f32, gain: f32, base_freq: f32) -> f32 {
    let mut freq = base_freq;
    let mut amp = 1.0_f32;
    let mut sum = 0.0_f32;
    let mut norm = 0.0_f32;
    for o in 0..octaves {
        sum += amp * value_noise(pos * freq, seed.wrapping_add(o.wrapping_mul(0x9E37_79B1)));
        norm += amp;
        freq *= lacunarity;
        amp *= gain;
    }
    (sum / norm).clamp(0.0, 1.0)
}
