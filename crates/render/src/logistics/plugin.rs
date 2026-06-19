use bevy::app::{App, Plugin, Update};
use bevy::prelude::*;

use bevy::sprite_render::AlphaMode2d;
use hitboxes_rapier::components::Collider;
use logistics::cargo_handling::components::DockZone;
use logistics::components::Storage;
use sprites::components::SpriteRef;

use crate::logistics::constants::{DOCK_ZONE_COLOR, DOCK_ZONE_Z_OFFSET, STORAGE_COLOR};
use crate::obstacle::mesh::shape_mesh;

pub struct StorageRenderPlugin;

impl Plugin for StorageRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (attach_storage_mesh, attach_dock_zone_mesh));
    }
}

/// Newly-added storage that isn't skinned by a sprite — those get the procedural square mesh.
/// Aliased to keep the query type under clippy's complexity bar.
type UntexturedStorage = (Added<Storage>, Without<SpriteRef>);

/// Give each newly-added storage building a square mesh (from its collider geometry) and the storage
/// material. One-shot at spawn — mirrors `attach_obstacle_mesh`, reusing its `shape_mesh`.
fn attach_storage_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    query: Query<(Entity, &Collider), UntexturedStorage>,
) {
    for (entity, collider) in &query {
        let mesh = meshes.add(shape_mesh(&collider.shape));
        commands.entity(entity).insert((
            Mesh2d(mesh),
            MeshMaterial2d(materials.add(ColorMaterial::from(STORAGE_COLOR))),
        ));
    }
}

/// Give each newly-added [`DockZone`] a faint translucent disc child showing its load/unload range.
/// A child so it tracks the holder; offset below it so the building still reads on top.
fn attach_dock_zone_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    query: Query<(Entity, &DockZone), Added<DockZone>>,
) {
    for (entity, zone) in &query {
        let mesh = meshes.add(Circle::new(zone.radius));
        let material = materials.add(ColorMaterial {
            color: DOCK_ZONE_COLOR,
            alpha_mode: AlphaMode2d::Blend,
            ..default()
        });
        commands.entity(entity).with_child((
            Mesh2d(mesh),
            MeshMaterial2d(material),
            Transform::from_xyz(0.0, 0.0, DOCK_ZONE_Z_OFFSET),
        ));
    }
}
