use bevy::math::Vec2;

use crate::elevation::noise::value_noise::value_noise;

#[must_use]
pub fn fbm(pos: Vec2, seed: u32, octaves: u32, lacunarity: f32, gain: f32, base_freq: f32) -> f32 {
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
