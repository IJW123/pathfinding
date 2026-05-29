use bevy::math::Vec2;

#[must_use]
pub fn value_noise(p: Vec2, seed: u32) -> f32 {
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
