use bevy::app::{App, Plugin, Update};
use bevy::ecs::query::QueryData;
use bevy::prelude::*;

use hitboxes_rapier::components::{Collider, Static};
use obstacles::components::{Obstacle, Wall};

use crate::obstacle::constants::{
    OBSTACLE_DYNAMIC_COLOR, OBSTACLE_STATIC_COLOR, OBSTACLE_WALL_COLOR,
};
use crate::obstacle::mesh::shape_mesh;

pub struct ObstacleRenderPlugin;

impl Plugin for ObstacleRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, attach_obstacle_mesh);
    }
}

/// What the renderer reads off each newly-added obstacle. A named query keeps the system signature
/// readable as the per-kind discriminators grow (`Static`, `Wall`, ...).
#[derive(QueryData)]
struct ObstacleRender {
    entity: Entity,
    collider: &'static Collider,
    is_static: Option<&'static Static>,
    is_wall: Option<&'static Wall>,
}

/// Per-kind fill color: boundary wall takes precedence over static, which takes precedence over
/// pushable.
fn obstacle_color(is_wall: bool, is_static: bool) -> Color {
    match (is_wall, is_static) {
        (true, _) => OBSTACLE_WALL_COLOR,
        (false, true) => OBSTACLE_STATIC_COLOR,
        (false, false) => OBSTACLE_DYNAMIC_COLOR,
    }
}

/// Give each newly-added obstacle a mesh (from its collider geometry) and a kind-colored material.
/// One-shot at spawn — doesn't track later `Static`/shape changes.
fn attach_obstacle_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    query: Query<ObstacleRender, Added<Obstacle>>,
) {
    for item in &query {
        let color = obstacle_color(item.is_wall.is_some(), item.is_static.is_some());
        let mesh = meshes.add(shape_mesh(&item.collider.shape));
        commands.entity(item.entity).insert((
            Mesh2d(mesh),
            MeshMaterial2d(materials.add(ColorMaterial::from(color))),
        ));
    }
}
