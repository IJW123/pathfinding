use bevy::math::Vec2;

use crate::elevation::noise::hash::hash_to_unit;

#[must_use]
pub fn value_noise(p: Vec2, seed: u32) -> f32 {
    let ix = p.x.floor() as i32;
    let iy = p.y.floor() as i32;
    let fx = p.x - ix as f32;
    let fy = p.y - iy as f32;

    let v00 = hash_to_unit(ix, iy, seed);
    let v10 = hash_to_unit(ix + 1, iy, seed);
    let v01 = hash_to_unit(ix, iy + 1, seed);
    let v11 = hash_to_unit(ix + 1, iy + 1, seed);

    let ux = smoothstep(fx);
    let uy = smoothstep(fy);

    let a = v00 + (v10 - v00) * ux;
    let b = v01 + (v11 - v01) * ux;
    a + (b - a) * uy
}

fn smoothstep(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f32 = 1e-4;

    fn approx(a: f32, b: f32) -> bool {
        (a - b).abs() < EPS
    }

    #[test]
    fn equals_corner_hash_at_lattice_nodes() {
        // fx == fy == 0 ⇒ no blend, value collapses to the v00 corner sample.
        let seed = 0xC0FFEE;
        for iy in -3..=3 {
            for ix in -3..=3 {
                let p = Vec2::new(ix as f32, iy as f32);
                let expected = hash_to_unit(ix, iy, seed);
                assert!(
                    approx(value_noise(p, seed), expected),
                    "lattice node ({ix},{iy}) should equal its corner hash"
                );
            }
        }
    }

    #[test]
    fn stays_in_unit_interval() {
        let seed = 7;
        for sy in 0..20 {
            for sx in 0..20 {
                let p = Vec2::new(sx as f32 * 0.37 - 3.0, sy as f32 * 0.41 - 3.0);
                let v = value_noise(p, seed);
                assert!((0.0..=1.0).contains(&v), "value_noise out of range: {v}");
            }
        }
    }

    #[test]
    fn is_deterministic() {
        let p = Vec2::new(1.25, -2.75);
        assert_eq!(value_noise(p, 99), value_noise(p, 99));
    }
}
