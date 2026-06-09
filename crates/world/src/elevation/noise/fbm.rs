use bevy::math::Vec2;

use crate::elevation::noise::value_noise::value_noise;

/// `octaves` must be >= 1; with zero octaves the normalisation accumulator stays zero and
/// the result is `NaN` (0/0).
#[must_use]
pub fn fbm(pos: Vec2, seed: u32, octaves: u32, lacunarity: f32, gain: f32, base_freq: f32) -> f32 {
    debug_assert!(octaves >= 1, "fbm requires at least one octave");
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elevation::noise::value_noise::value_noise;

    const EPS: f32 = 1e-4;

    fn approx(a: f32, b: f32) -> bool {
        (a - b).abs() < EPS
    }

    #[test]
    fn single_octave_is_scaled_value_noise() {
        // octave 0 adds seed.wrapping_add(0) == seed; norm == amp == 1.
        let p = Vec2::new(12.5, -8.0);
        let (seed, bf) = (0xC0FFEE, 1.0 / 50.0);
        let expected = value_noise(p * bf, seed).clamp(0.0, 1.0);
        assert!(approx(fbm(p, seed, 1, 2.0, 0.5, bf), expected));
    }

    #[test]
    fn stays_in_unit_interval() {
        for sy in 0..15 {
            for sx in 0..15 {
                let p = Vec2::new(sx as f32 * 13.0, sy as f32 * 17.0);
                let v = fbm(p, 1, 4, 2.0, 0.5, 1.0 / 200.0);
                assert!((0.0..=1.0).contains(&v), "fbm out of range: {v}");
            }
        }
    }

    #[test]
    fn is_deterministic() {
        let p = Vec2::new(33.0, 44.0);
        assert_eq!(fbm(p, 5, 4, 2.0, 0.5, 0.01), fbm(p, 5, 4, 2.0, 0.5, 0.01));
    }

    #[test]
    fn base_frequency_changes_output() {
        let p = Vec2::new(100.0, 100.0);
        assert_ne!(fbm(p, 5, 4, 2.0, 0.5, 0.005), fbm(p, 5, 4, 2.0, 0.5, 0.05));
    }
}
