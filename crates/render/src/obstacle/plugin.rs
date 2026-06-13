use bevy::app::{App, Plugin, Update};
use bevy::prelude::*;

use hitboxes_rapier::components::{Collider, Static};
use hitboxes_rapier::shape::ColliderShape;
use obstacles::components::Obstacle;

use crate::obstacle::constants::{OBSTACLE_DYNAMIC_COLOR, OBSTACLE_STATIC_COLOR};
use crate::obstacle::mesh::convex_mesh;

pub struct ObstacleRenderPlugin;

impl Plugin for ObstacleRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, attach_obstacle_mesh);
    }
}

/// Build each new obstacle's mesh straight from its collider shape — single source of truth,
/// so visuals can't drift from collision geometry. Color marks static vs pushable (one-shot
/// decision at spawn; doesn't track later `Static` changes).
fn attach_obstacle_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    query: Query<(Entity, &Collider, Option<&Static>), Added<Obstacle>>,
) {
    for (entity, collider, is_static) in &query {
        let color = if is_static.is_some() {
            OBSTACLE_STATIC_COLOR
        } else {
            OBSTACLE_DYNAMIC_COLOR
        };
        let mesh = match &collider.shape {
            ColliderShape::Circle(ball) => meshes.add(Circle::new(ball.radius)),
            ColliderShape::Convex(_) => {
                let points = collider.shape.hull_points().expect("convex shape");
                meshes.add(convex_mesh(&points))
            }
            ColliderShape::Obb(_) => {
                let size = collider.shape.local_extent();
                meshes.add(Rectangle::new(size.x, size.y))
            }
        };
        commands.entity(entity).insert((
            Mesh2d(mesh),
            MeshMaterial2d(materials.add(ColorMaterial::from(color))),
        ));
    }
}
