use bevy::math::{IVec2, Vec2};

use crate::elevation::chunk_coord::chunk_origin_world;
use crate::elevation::constants::{ELEV_CHUNK_CELLS, ELEVATION_CELL};
use crate::elevation::contour::data::{ContourLine, ContourSegment};
use crate::elevation::contour::marching::emit_cell_segments;
use crate::elevation::height_field::HeightField;

#[must_use]
pub fn extract_contours(coord: IVec2, height: &HeightField, levels: &[f32]) -> Vec<ContourLine> {
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
            ContourLine {
                level: iso,
                segments,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::math::UVec2;

    const DIM: usize = ELEV_CHUNK_CELLS + 1; // 33 nodes per axis
    const SPAN: f32 = ELEV_CHUNK_CELLS as f32 * ELEVATION_CELL; // 320

    /// A height field exactly covering one chunk at `coord`, ramping as `world_x - origin.x`
    /// (i.e. local height == column index * cell), so contour levels map to predictable x.
    fn ramp_field(coord: IVec2) -> HeightField {
        let origin = chunk_origin_world(coord);
        let mut data = vec![0.0_f32; DIM * DIM];
        for j in 0..DIM {
            for i in 0..DIM {
                data[j * DIM + i] = i as f32 * ELEVATION_CELL;
            }
        }
        HeightField::from_parts(UVec2::splat(DIM as u32), origin, ELEVATION_CELL, data)
    }

    fn flat_field(value: f32) -> HeightField {
        HeightField::from_parts(
            UVec2::splat(DIM as u32),
            chunk_origin_world(IVec2::ZERO),
            ELEVATION_CELL,
            vec![value; DIM * DIM],
        )
    }

    #[test]
    fn returns_one_line_per_level_in_order() {
        let field = ramp_field(IVec2::ZERO);
        let levels = [50.0, 160.0, 250.0];
        let lines = extract_contours(IVec2::ZERO, &field, &levels);
        assert_eq!(lines.len(), levels.len());
        for (line, &level) in lines.iter().zip(&levels) {
            assert_eq!(line.level, level);
        }
    }

    #[test]
    fn ramp_contour_is_vertical_at_expected_x() {
        let field = ramp_field(IVec2::ZERO);
        // height == local_x, so level 160 ⇒ contour at local x = 160.
        let lines = extract_contours(IVec2::ZERO, &field, &[160.0]);
        let segs = &lines[0].segments;
        assert!(
            !segs.is_empty(),
            "ramp must produce a contour at the mid level"
        );
        for s in segs {
            assert!((s.a.x - 160.0).abs() < 1e-3);
            assert!((s.b.x - 160.0).abs() < 1e-3);
        }
    }

    #[test]
    fn flat_field_outside_levels_emits_no_segments() {
        let field = flat_field(100.0);
        let lines = extract_contours(IVec2::ZERO, &field, &[10.0, 300.0]);
        assert_eq!(lines.len(), 2);
        assert!(lines.iter().all(|l| l.segments.is_empty()));
    }

    #[test]
    fn segments_are_chunk_local_regardless_of_coord() {
        let coord = IVec2::new(2, 3);
        let field = ramp_field(coord);
        let lines = extract_contours(coord, &field, &[160.0]);
        let segs = &lines[0].segments;
        assert!(!segs.is_empty());
        for s in segs {
            for p in [s.a, s.b] {
                assert!((0.0..=SPAN).contains(&p.x), "x not chunk-local: {}", p.x);
                assert!((0.0..=SPAN).contains(&p.y), "y not chunk-local: {}", p.y);
            }
        }
    }
}
