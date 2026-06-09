use bevy::math::{IVec2, Vec2};

use crate::constants::MAP_HALF_EXTENT;
use crate::elevation::constants::{ELEV_CHUNK_CELLS, ELEVATION_CELL};

#[must_use]
pub fn chunk_origin_world(coord: IVec2) -> Vec2 {
    let span = ELEV_CHUNK_CELLS as f32 * ELEVATION_CELL;
    Vec2::new(coord.x as f32 * span, coord.y as f32 * span)
}

/// Every chunk coord whose tile overlaps the fixed map, in row-major order. The map is
/// static, so this is the full, immutable set of terrain tiles — enumerated once at launch.
#[must_use]
pub fn map_chunk_coords() -> Vec<IVec2> {
    let span = ELEV_CHUNK_CELLS as f32 * ELEVATION_CELL;
    let min = (-MAP_HALF_EXTENT / span).floor() as i32;
    let max = (MAP_HALF_EXTENT / span).floor() as i32;
    (min..=max)
        .flat_map(|cy| (min..=max).map(move |cx| IVec2::new(cx, cy)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SPAN: f32 = ELEV_CHUNK_CELLS as f32 * ELEVATION_CELL; // 320

    #[test]
    fn origin_scales_by_chunk_span() {
        assert_eq!(chunk_origin_world(IVec2::ZERO), Vec2::ZERO);
        assert_eq!(chunk_origin_world(IVec2::new(1, 0)), Vec2::new(SPAN, 0.0));
        assert_eq!(
            chunk_origin_world(IVec2::new(-2, 3)),
            Vec2::new(-2.0 * SPAN, 3.0 * SPAN)
        );
    }

    #[test]
    fn map_chunk_coords_covers_full_map_row_major() {
        // min = floor(-2000/320) = -7, max = floor(2000/320) = 6 ⇒ 14 per axis.
        let coords = map_chunk_coords();
        assert_eq!(coords.len(), 14 * 14);
        assert_eq!(coords[0], IVec2::new(-7, -7)); // outer cy, inner cx
        assert_eq!(coords[1], IVec2::new(-6, -7));
        assert_eq!(*coords.last().unwrap(), IVec2::new(6, 6));
    }
}
