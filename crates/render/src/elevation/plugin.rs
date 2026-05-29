use bevy::app::{App, Plugin, Update};
use bevy::prelude::IntoScheduleConfigs;

use world::elevation::plugin::ElevationLifecycleSet;

use crate::elevation::contour::levels::ContourLevels;
use crate::elevation::contour::render::render_contours_on_chunk_loaded;
use crate::elevation::contour::style::ContourStyle;

pub struct ContourRenderPlugin;

impl Plugin for ContourRenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ContourLevels>()
            .init_resource::<ContourStyle>()
            .add_systems(
                Update,
                render_contours_on_chunk_loaded.after(ElevationLifecycleSet),
            );
    }
}
