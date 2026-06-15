use bevy::app::{App, Plugin, PreStartup};
use bevy::prelude::*;

use crate::elevation::config::TerrainConfig;
use crate::elevation::height_field::HeightField;

pub struct ElevationPlugin;

impl Plugin for ElevationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, build_height_field);
    }
}

/// Build the immutable [`HeightField`] from the level-authored [`TerrainConfig`] before any
/// `Startup` consumer runs. Panics if no `TerrainConfig` was inserted — map content must be
/// provided by the `level` crate; `world` has no default recipe.
fn build_height_field(mut commands: Commands, config: Res<TerrainConfig>) {
    commands.insert_resource(HeightField::new(&config));
}
