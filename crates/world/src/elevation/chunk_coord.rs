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
