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
    FLAT_AMP * fbm(p, FBM_SEED, FBM_OCTAVES, FBM_LACUNARITY, FBM_GAIN, FLAT_FREQ)
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
    let detail =
        1.0 + HILL_ROUGHNESS * (fbm(p, FBM_SEED, FBM_OCTAVES, FBM_LACUNARITY, FBM_GAIN, HILL_DETAIL_FREQ) - 0.5);
    spec.height * falloff * detail
}
