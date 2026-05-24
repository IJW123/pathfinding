use bevy::asset::RenderAssetUsages;
use bevy::color::{Color, ColorToComponents, LinearRgba};
use bevy::math::{IVec2, Vec2};
use bevy::mesh::{Mesh, PrimitiveTopology};

use crate::constants::{CONTOUR_STEP, ELEVATION_CELL, ELEV_CHUNK_CELLS, HEIGHT_MAX, HEIGHT_MIN};
use crate::world::elevation::height_fn::HeightFn;
use crate::world::elevation::marching::emit_cell_segments;

#[must_use]
pub fn chunk_origin_world(coord: IVec2) -> Vec2 {
    let span = ELEV_CHUNK_CELLS as f32 * ELEVATION_CELL;
    Vec2::new(coord.x as f32 * span, coord.y as f32 * span)
}

#[must_use]
pub fn build_chunk_mesh(coord: IVec2, height: &HeightFn) -> Mesh {
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

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut colors: Vec<[f32; 4]> = Vec::new();

    let levels = contour_levels();
    for (idx, &iso) in levels.iter().enumerate() {
        let color = level_color(idx, levels.len());
        let rgba = LinearRgba::from(color).to_f32_array();

        for cy in 0..ELEV_CHUNK_CELLS {
            for cx in 0..ELEV_CHUNK_CELLS {
                let c0 = samples[cy * dim + cx];
                let c1 = samples[cy * dim + (cx + 1)];
                let c2 = samples[(cy + 1) * dim + (cx + 1)];
                let c3 = samples[(cy + 1) * dim + cx];
                let bl = Vec2::new(cx as f32 * step, cy as f32 * step);

                emit_cell_segments(bl, step, [c0, c1, c2, c3], iso, &mut |a, b| {
                    positions.push([a.x, a.y, 0.0]);
                    positions.push([b.x, b.y, 0.0]);
                    colors.push(rgba);
                    colors.push(rgba);
                });
            }
        }
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::LineList,
        RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh
}

fn contour_levels() -> Vec<f32> {
    let range = HEIGHT_MAX - HEIGHT_MIN;
    let count = (range / CONTOUR_STEP).floor() as i32 - 1;
    (0..count.max(0))
        .map(|i| HEIGHT_MIN + (i as f32 + 1.0) * CONTOUR_STEP)
        .collect()
}

fn level_color(idx: usize, total: usize) -> Color {
    let t = if total <= 1 {
        0.5
    } else {
        idx as f32 / (total - 1) as f32
    };
    let low = Vec2::new(0.4, 0.25);
    let high = Vec2::new(0.95, 0.9);
    let r = low.x + (high.x - low.x) * t;
    let g = low.y + (high.y - low.y) * t;
    let b = 0.1 + (0.8 - 0.1) * t;
    Color::srgb(r, g, b)
}
