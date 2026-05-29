use bevy::math::{IVec2, Vec2};

use crate::elevation::constants::{ELEVATION_CELL, ELEV_CHUNK_CELLS};

#[must_use]
pub fn chunk_origin_world(coord: IVec2) -> Vec2 {
    let span = ELEV_CHUNK_CELLS as f32 * ELEVATION_CELL;
    Vec2::new(coord.x as f32 * span, coord.y as f32 * span)
}
