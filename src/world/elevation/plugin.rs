use bevy::app::{App, Plugin, Update};

use crate::world::elevation::contour_levels::ContourLevels;
use crate::world::elevation::contour_style::ContourStyle;
use crate::world::elevation::height_fn::HeightFn;
use crate::world::elevation::resources::LoadedChunks;
use crate::world::elevation::streaming::update_visible_chunks;

pub struct ElevationPlugin;

impl Plugin for ElevationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HeightFn>()
            .init_resource::<ContourLevels>()
            .init_resource::<ContourStyle>()
            .init_resource::<LoadedChunks>()
            .add_systems(Update, update_visible_chunks);
    }
}
