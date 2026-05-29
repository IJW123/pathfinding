use bevy::prelude::Resource;

use world::elevation::constants::{HEIGHT_MAX, HEIGHT_MIN};

use crate::elevation::contour::constants::CONTOUR_STEP;

#[derive(Resource)]
pub struct ContourLevels(pub Vec<f32>);

impl Default for ContourLevels {
    fn default() -> Self {
        let range = HEIGHT_MAX - HEIGHT_MIN;
        let count = (range / CONTOUR_STEP).floor() as i32 - 1;
        let levels = (0..count.max(0))
            .map(|i| HEIGHT_MIN + (i as f32 + 1.0) * CONTOUR_STEP)
            .collect();
        Self(levels)
    }
}
