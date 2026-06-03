use bevy::app::{App, Plugin, Startup};

use crate::elevation::contour::levels::ContourLevels;
use crate::elevation::contour::render::spawn_contour_tiles;
use crate::elevation::contour::style::ContourStyle;

pub struct ContourRenderPlugin;

impl Plugin for ContourRenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ContourLevels>()
            .init_resource::<ContourStyle>()
            .add_systems(Startup, spawn_contour_tiles);
    }
}
