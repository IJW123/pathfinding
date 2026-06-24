use bevy::math::{Rect, Vec2};
use routing::ElevationSampler;

use crate::elevation::height_field::HeightField;

/// Lets the `routing` crate read terrain off the live heightmap without depending on `world`'s
/// internals. Pure delegation to [`HeightField`]'s existing sampling and extent.
impl ElevationSampler for HeightField {
    fn height(&self, p: Vec2) -> f32 {
        self.sample(p)
    }

    fn bounds(&self) -> Rect {
        HeightField::bounds(self)
    }
}
