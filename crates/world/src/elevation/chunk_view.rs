use std::collections::HashSet;

use bevy::math::{IVec2, Vec2};

use crate::elevation::constants::{CHUNK_VIEW_MARGIN, ELEVATION_CELL, ELEV_CHUNK_CELLS};

#[must_use]
pub fn desired_chunks(cam_pos: Vec2, viewport_size: Vec2) -> HashSet<IVec2> {
    let half = viewport_size * 0.5 + Vec2::splat(CHUNK_VIEW_MARGIN);
    let min = cam_pos - half;
    let max = cam_pos + half;
    let span = ELEV_CHUNK_CELLS as f32 * ELEVATION_CELL;
    let min_cx = (min.x / span).floor() as i32;
    let max_cx = (max.x / span).floor() as i32;
    let min_cy = (min.y / span).floor() as i32;
    let max_cy = (max.y / span).floor() as i32;

    let mut out = HashSet::new();
    for cy in min_cy..=max_cy {
        for cx in min_cx..=max_cx {
            out.insert(IVec2::new(cx, cy));
        }
    }
    out
}
