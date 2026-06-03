use bevy::app::{App, Plugin};

use crate::elevation::height_field::HeightField;

pub struct ElevationPlugin;

impl Plugin for ElevationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HeightField>();
    }
}
