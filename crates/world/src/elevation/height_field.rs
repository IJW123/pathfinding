use bevy::math::{UVec2, Vec2};
use bevy::prelude::Resource;

use crate::constants::MAP_HALF_EXTENT;
use crate::elevation::constants::{ELEVATION_CELL, HEIGHT_MAX, HEIGHT_MIN};
use crate::elevation::generation::feature::{feature_value, flat_base};
use crate::elevation::generation::placement::all_features;

/// Stored 2D heightmap covering the finite map. Generated once at startup (flat base +
/// stamped hill/mountain features) and immutable thereafter, so the per-chunk contour
/// cache stays valid. Sampled by bilinear interpolation; grid nodes are spaced
/// [`ELEVATION_CELL`] apart and anchored at `-MAP_HALF_EXTENT`.
#[derive(Resource)]
pub struct HeightField {
    dims: UVec2,
    origin: Vec2,
    cell: f32,
    data: Vec<f32>,
}

impl Default for HeightField {
    fn default() -> Self {
        let cell = ELEVATION_CELL;
        let n = (2.0 * MAP_HALF_EXTENT / cell).round() as u32 + 1;
        let dims = UVec2::splat(n);
        let origin = Vec2::splat(-MAP_HALF_EXTENT);
        let features = all_features();

        let mut data = Vec::with_capacity((n * n) as usize);
        for j in 0..n {
            for i in 0..n {
                let p = origin + Vec2::new(i as f32 * cell, j as f32 * cell);
                let height = features
                    .iter()
                    .fold(flat_base(p), |acc, f| acc + feature_value(p, f));
                data.push(height.clamp(HEIGHT_MIN, HEIGHT_MAX));
            }
        }

        Self {
            dims,
            origin,
            cell,
            data,
        }
    }
}

impl HeightField {
    fn at(&self, i: u32, j: u32) -> f32 {
        self.data[(j * self.dims.x + i) as usize]
    }

    /// Bilinearly interpolated height at a world position, clamped to the map edge.
    #[must_use]
    pub fn sample(&self, pos: Vec2) -> f32 {
        let max_x = (self.dims.x - 1) as f32;
        let max_y = (self.dims.y - 1) as f32;
        let local = (pos - self.origin) / self.cell;
        let gx = local.x.clamp(0.0, max_x);
        let gy = local.y.clamp(0.0, max_y);

        let i0 = gx.floor() as u32;
        let j0 = gy.floor() as u32;
        let i1 = (i0 + 1).min(self.dims.x - 1);
        let j1 = (j0 + 1).min(self.dims.y - 1);
        let fx = gx - i0 as f32;
        let fy = gy - j0 as f32;

        let a = self.at(i0, j0) + (self.at(i1, j0) - self.at(i0, j0)) * fx;
        let b = self.at(i0, j1) + (self.at(i1, j1) - self.at(i0, j1)) * fx;
        a + (b - a) * fy
    }

    /// Central-difference gradient of the height field (world units per world unit).
    #[must_use]
    pub fn gradient(&self, pos: Vec2) -> Vec2 {
        let eps = self.cell;
        let dx = self.sample(pos + Vec2::new(eps, 0.0)) - self.sample(pos - Vec2::new(eps, 0.0));
        let dy = self.sample(pos + Vec2::new(0.0, eps)) - self.sample(pos - Vec2::new(0.0, eps));
        Vec2::new(dx, dy) / (2.0 * eps)
    }
}
