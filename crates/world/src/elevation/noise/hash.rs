/// Deterministic integer hash of a 2D lattice coord + seed, used by value noise
/// and seeded feature placement. No randomness source — same inputs, same output.
#[must_use]
pub fn hash_u32(ix: i32, iy: i32, seed: u32) -> u32 {
    let mut h =
        seed ^ (ix as u32).wrapping_mul(0x9E37_79B1) ^ (iy as u32).wrapping_mul(0x85EB_CA77);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_u32_is_deterministic() {
        assert_eq!(hash_u32(3, -7, 42), hash_u32(3, -7, 42));
        assert_eq!(hash_u32(0, 0, 0), hash_u32(0, 0, 0));
    }

    #[test]
    fn hash_u32_is_sensitive_to_each_input() {
        let base = hash_u32(3, -7, 42);
        assert_ne!(base, hash_u32(4, -7, 42), "ix change must alter hash");
        assert_ne!(base, hash_u32(3, -6, 42), "iy change must alter hash");
        assert_ne!(base, hash_u32(3, -7, 43), "seed change must alter hash");
    }

    #[test]
    fn hash_to_unit_stays_in_unit_interval() {
        for seed in [0u32, 1, 0xC0FFEE, u32::MAX] {
            for iy in -5..=5 {
                for ix in -5..=5 {
                    let v = hash_to_unit(ix, iy, seed);
                    assert!((0.0..=1.0).contains(&v), "hash_to_unit out of range: {v}");
                }
            }
        }
    }

    #[test]
    fn hash_to_unit_reseeds() {
        assert_ne!(hash_to_unit(2, 2, 1), hash_to_unit(2, 2, 2));
    }
}
