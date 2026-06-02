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
