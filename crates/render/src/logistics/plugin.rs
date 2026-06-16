use bevy::app::{App, Plugin, Update};
use bevy::prelude::*;

use hitboxes_rapier::components::Collider;
use logistics::components::Storage;

use crate::logistics::constants::STORAGE_COLOR;
use crate::obstacle::mesh::shape_mesh;

pub struct StorageRenderPlugin;

impl Plugin for StorageRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, attach_storage_mesh);
    }
}

/// Give each newly-added storage building a square mesh (from its collider geometry) and the storage
/// material. One-shot at spawn — mirrors `attach_obstacle_mesh`, reusing its `shape_mesh`.
fn attach_storage_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    query: Query<(Entity, &Collider), Added<Storage>>,
) {
    for (entity, collider) in &query {
        let mesh = meshes.add(shape_mesh(&collider.shape));
        commands.entity(entity).insert((
            Mesh2d(mesh),
            MeshMaterial2d(materials.add(ColorMaterial::from(STORAGE_COLOR))),
        ));
    }
}
