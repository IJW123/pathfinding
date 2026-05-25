use bevy::prelude::Component;

use crate::world::elevation::contour_data::ContourLine;

#[derive(Component)]
pub struct ElevationChunk;

#[derive(Component)]
pub struct ContourGeometry {
    #[expect(dead_code, reason = "queried by upcoming pathfinding/overlay systems")]
    pub lines: Vec<ContourLine>,
}
