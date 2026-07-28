use bevy::app::{App, Plugin, Update};
use bevy::prelude::*;

use station::components::Station;

use crate::station::constants::{PLATFORM_LENGTH, PLATFORM_WIDTH, STATION_COLOR};

pub struct StationRenderPlugin;

impl Plugin for StationRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, attach_station_platform);
    }
}

/// Give each newly-added station a thick platform-bar mesh. The bar is a rectangle `PLATFORM_LENGTH`
/// along the track by `PLATFORM_WIDTH` across; the station's transform (position + the along-track
/// rotation the spawner set) places and orients it. One-shot at spawn — stations don't move.
fn attach_station_platform(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    query: Query<Entity, Added<Station>>,
) {
    for entity in &query {
        let mesh = meshes.add(Rectangle::new(PLATFORM_LENGTH, PLATFORM_WIDTH));
        commands.entity(entity).insert((
            Mesh2d(mesh),
            MeshMaterial2d(materials.add(ColorMaterial::from(STATION_COLOR))),
        ));
    }
}
