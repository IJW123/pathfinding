use bevy::app::{App, Plugin, Update};
use bevy::prelude::*;

use hitboxes_rapier::components::Collider;
use selection::components::Selected;

use crate::obstacle::mesh::shape_mesh;
use crate::selection::components::HighlightMarker;
use crate::selection::constants::{HIGHLIGHT_COLOR, HIGHLIGHT_SCALE, HIGHLIGHT_Z_OFFSET};

pub struct SelectionRenderPlugin;

impl Plugin for SelectionRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (attach_highlight, clear_highlight));
    }
}

/// On selection, give the entity an outline child: its collider silhouette scaled up and tinted,
/// sitting just behind it so the edge reads as a border ring. Reuses `shape_mesh` so the highlight
/// can't drift from the collider.
fn attach_highlight(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    query: Query<(Entity, &Collider), Added<Selected>>,
) {
    for (entity, collider) in &query {
        let mesh = meshes.add(shape_mesh(&collider.shape));
        commands.entity(entity).with_child((
            Mesh2d(mesh),
            MeshMaterial2d(materials.add(ColorMaterial::from(HIGHLIGHT_COLOR))),
            Transform::from_xyz(0.0, 0.0, HIGHLIGHT_Z_OFFSET).with_scale(Vec3::new(
                HIGHLIGHT_SCALE,
                HIGHLIGHT_SCALE,
                1.0,
            )),
            HighlightMarker,
        ));
    }
}

/// On deselection, despawn the holder's highlight child. We walk the surviving highlight children
/// and match them to the deselected holders by `ChildOf` — a child whose holder was despawned is
/// already gone with it, so it never shows up here (no stale-entity guard needed).
fn clear_highlight(
    mut commands: Commands,
    mut deselected: RemovedComponents<Selected>,
    highlights: Query<(Entity, &ChildOf), With<HighlightMarker>>,
) {
    let holders: Vec<Entity> = deselected.read().collect();
    for (child, child_of) in &highlights {
        if holders.contains(&child_of.parent()) {
            commands.entity(child).despawn();
        }
    }
}
