use bevy::math::{UVec2, Vec2};
use bevy::prelude::Resource;

use crate::elevation::config::TerrainConfig;
use crate::elevation::constants::{ELEVATION_CELL, HEIGHT_MAX, HEIGHT_MIN};
use crate::elevation::generation::feature::{feature_value, flat_base};
use crate::elevation::generation::placement::all_features;

/// Stored 2D heightmap covering the finite map. Generated once at startup (flat base +
/// stamped hill/mountain features) and immutable thereafter, so the per-chunk contour
/// cache stays valid. Sampled by bilinear interpolation; grid nodes are spaced
/// [`ELEVATION_CELL`] apart and anchored at `-half_extent`.
#[derive(Resource)]
pub struct HeightField {
    dims: UVec2,
    origin: Vec2,
    cell: f32,
    data: Vec<f32>,
}

impl HeightField {
    /// Build the heightmap from a level's [`TerrainConfig`]: a flat base with every recipe feature
    /// stamped on top, clamped to `[HEIGHT_MIN, HEIGHT_MAX]`.
    #[must_use]
    pub fn new(config: &TerrainConfig) -> Self {
        let cell = ELEVATION_CELL;
        let n = (2.0 * config.half_extent / cell).round() as u32 + 1;
        let dims = UVec2::splat(n);
        let origin = Vec2::splat(-config.half_extent);
        let features = all_features(config);

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

    /// Build a height field from explicit parts, bypassing feature generation. Test seam
    /// for constructing known fields (e.g. a linear ramp) with predictable sampling.
    #[cfg(test)]
    pub(crate) fn from_parts(dims: UVec2, origin: Vec2, cell: f32, data: Vec<f32>) -> Self {
        assert_eq!(
            data.len(),
            (dims.x * dims.y) as usize,
            "data must match dims"
        );
        Self {
            dims,
            origin,
            cell,
            data,
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f32 = 1e-3;

    fn approx(a: f32, b: f32) -> bool {
        (a - b).abs() < EPS
    }

    /// 5x5 field, cell 10, origin (0,0). Height ramps as `2 * x` (column-driven), so each
    /// row is [0, 20, 40, 60, 80]. Wide enough that a central-difference gradient (eps = cell)
    /// stays interior away from the clamp edges.
    fn ramp_x() -> HeightField {
        let row = [0.0, 20.0, 40.0, 60.0, 80.0];
        let data: Vec<f32> = (0..5).flat_map(|_| row).collect();
        HeightField::from_parts(UVec2::splat(5), Vec2::ZERO, 10.0, data)
    }

    #[test]
    fn samples_exactly_at_grid_nodes() {
        let f = ramp_x();
        assert!(approx(f.sample(Vec2::new(0.0, 0.0)), 0.0));
        assert!(approx(f.sample(Vec2::new(10.0, 0.0)), 20.0));
        assert!(approx(f.sample(Vec2::new(20.0, 20.0)), 40.0));
    }

    #[test]
    fn bilinear_midpoint_is_node_average() {
        let f = ramp_x();
        // halfway between the x=0 (0) and x=10 (20) columns ⇒ 10.
        assert!(approx(f.sample(Vec2::new(5.0, 0.0)), 10.0));
        assert!(approx(f.sample(Vec2::new(15.0, 12.0)), 30.0));
    }

    #[test]
    fn out_of_bounds_clamps_to_edge() {
        let f = ramp_x();
        assert!(approx(f.sample(Vec2::new(-100.0, 0.0)), 0.0));
        assert!(approx(f.sample(Vec2::new(999.0, 0.0)), 80.0));
        assert!(approx(f.sample(Vec2::new(5.0, -50.0)), 10.0));
    }

    #[test]
    fn gradient_tracks_ramp_slope() {
        let f = ramp_x();
        let g = f.gradient(Vec2::new(20.0, 20.0)); // interior, away from clamp edges
        assert!(approx(g.x, 2.0), "expected slope 2 in x, got {}", g.x);
        assert!(approx(g.y, 0.0), "expected flat in y, got {}", g.y);
    }

    #[test]
    fn gradient_is_zero_on_flat_field() {
        let f = HeightField::from_parts(UVec2::splat(3), Vec2::ZERO, 10.0, vec![5.0; 9]);
        let g = f.gradient(Vec2::new(10.0, 10.0));
        assert!(approx(g.x, 0.0) && approx(g.y, 0.0));
    }
}
