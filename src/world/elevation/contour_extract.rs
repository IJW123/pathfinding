use bevy::math::{IVec2, Vec2};

use crate::constants::{ELEVATION_CELL, ELEV_CHUNK_CELLS};
use crate::world::elevation::chunk_coord::chunk_origin_world;
use crate::world::elevation::contour_data::{ContourLine, ContourSegment};
use crate::world::elevation::height_fn::HeightFn;
use crate::world::elevation::marching::emit_cell_segments;

#[must_use]
pub fn extract_contours(coord: IVec2, height: &HeightFn, levels: &[f32]) -> Vec<ContourLine> {
    let step = ELEVATION_CELL;
    let origin = chunk_origin_world(coord);
    let dim = ELEV_CHUNK_CELLS + 1;

    let mut samples = vec![0.0_f32; dim * dim];
    for j in 0..dim {
        for i in 0..dim {
            let world = origin + Vec2::new(i as f32 * step, j as f32 * step);
            samples[j * dim + i] = height.sample(world);
        }
    }

    levels
        .iter()
        .map(|&iso| {
            let mut segments = Vec::new();
            for cy in 0..ELEV_CHUNK_CELLS {
                for cx in 0..ELEV_CHUNK_CELLS {
                    let c0 = samples[cy * dim + cx];
                    let c1 = samples[cy * dim + (cx + 1)];
                    let c2 = samples[(cy + 1) * dim + (cx + 1)];
                    let c3 = samples[(cy + 1) * dim + cx];
                    let bl = Vec2::new(cx as f32 * step, cy as f32 * step);
                    emit_cell_segments(bl, step, [c0, c1, c2, c3], iso, &mut |a, b| {
                        segments.push(ContourSegment { a, b });
                    });
                }
            }
            ContourLine { level: iso, segments }
        })
        .collect()
}
