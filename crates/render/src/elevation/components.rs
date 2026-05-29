use bevy::prelude::Component;

use world::elevation::contour::data::ContourLine;

#[derive(Component)]
pub struct ContourGeometry {
    pub lines: Vec<ContourLine>,
}
