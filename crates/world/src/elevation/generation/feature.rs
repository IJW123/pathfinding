use bevy::math::Vec2;

use crate::elevation::constants::{
    FBM_GAIN, FBM_LACUNARITY, FBM_OCTAVES, FBM_SEED, FLAT_AMP, FLAT_FREQ, HILL_DETAIL_FREQ,
    HILL_ROUGHNESS,
};
use crate::elevation::noise::fbm::fbm;

/// A single elevation feature (hill or mountain) stamped onto the flat base.
#[derive(Clone, Copy)]
pub struct FeatureSpec {
    pub center: Vec2,
    pub radius: f32,
    pub height: f32,
}

/// Gentle, mostly-flat baseline so plains read as terrain rather than dead-level.
/// Stays low (`[0, FLAT_AMP]`), below the first contour level, so flats render empty.
#[must_use]
pub fn flat_base(p: Vec2) -> f32 {
    FLAT_AMP
        * fbm(
            p,
            FBM_SEED,
            FBM_OCTAVES,
            FBM_LACUNARITY,
            FBM_GAIN,
            FLAT_FREQ,
        )
}

/// Additive contribution of one feature at world point `p`: a smooth radial falloff
/// (1 at the center, 0 at the radius) perturbed by detail noise so the feature isn't a
/// perfect circular dome. Zero outside the radius.
#[must_use]
pub fn feature_value(p: Vec2, spec: &FeatureSpec) -> f32 {
    let t = (p.distance(spec.center) / spec.radius).clamp(0.0, 1.0);
    if t >= 1.0 {
        return 0.0;
    }
    let falloff = 1.0 - t * t * (3.0 - 2.0 * t);
    let detail = 1.0
        + HILL_ROUGHNESS
            * (fbm(
                p,
                FBM_SEED,
                FBM_OCTAVES,
                FBM_LACUNARITY,
                FBM_GAIN,
                HILL_DETAIL_FREQ,
            ) - 0.5);
    spec.height * falloff * detail
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec() -> FeatureSpec {
        FeatureSpec {
            center: Vec2::new(100.0, -50.0),
            radius: 200.0,
            height: 40.0,
        }
    }

    #[test]
    fn zero_at_and_beyond_radius() {
        let s = spec();
        let at_edge = s.center + Vec2::new(s.radius, 0.0);
        let outside = s.center + Vec2::new(s.radius * 2.0, 0.0);
        assert_eq!(feature_value(at_edge, &s), 0.0);
        assert_eq!(feature_value(outside, &s), 0.0);
    }

    #[test]
    fn center_value_within_detail_band() {
        let s = spec();
        // falloff == 1 at center; detail ∈ [1 - k/2, 1 + k/2].
        let band = HILL_ROUGHNESS * 0.5;
        let v = feature_value(s.center, &s);
        assert!(v >= s.height * (1.0 - band) - 1e-3);
        assert!(v <= s.height * (1.0 + band) + 1e-3);
    }

    #[test]
    fn inside_radius_is_bounded_and_positive() {
        let s = spec();
        let upper = s.height * (1.0 + HILL_ROUGHNESS * 0.5);
        for k in 0..8 {
            let p = s.center + Vec2::new(k as f32 * 20.0, 0.0); // all within radius
            let v = feature_value(p, &s);
            assert!(v > 0.0, "interior should be positive, got {v}");
            assert!(v <= upper + 1e-3, "exceeded falloff*detail bound: {v}");
        }
    }

    #[test]
    fn flat_base_stays_within_amplitude() {
        for sy in 0..12 {
            for sx in 0..12 {
                let p = Vec2::new(sx as f32 * 90.0, sy as f32 * 110.0);
                let v = flat_base(p);
                assert!((0.0..=FLAT_AMP).contains(&v), "flat_base out of range: {v}");
            }
        }
    }
}
