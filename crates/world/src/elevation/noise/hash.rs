/// Deterministic integer hash of a 2D lattice coord + seed, used by value noise
/// and seeded feature placement. No randomness source — same inputs, same output.
#[must_use]
pub fn hash_u32(ix: i32, iy: i32, seed: u32) -> u32 {
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

/// [`hash_u32`] mapped to the unit interval `[0, 1]`.
#[must_use]
pub fn hash_to_unit(ix: i32, iy: i32, seed: u32) -> f32 {
    (hash_u32(ix, iy, seed) as f32) / (u32::MAX as f32)
}
