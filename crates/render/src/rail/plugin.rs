use bevy::app::{App, Plugin, Update};
use bevy::prelude::*;

use hitboxes_rapier::components::Collider;
use rail::components::Locomotive;
use rail::track::RailTrack;

use crate::rail::constants::{LOCO_COLOR, RAIL_COLOR};
use crate::rail::mesh::track_line_mesh;

pub struct RailRenderPlugin;

impl Plugin for RailRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (attach_track_mesh, attach_locomotive_sprite));
    }
}

/// Give each newly-added track a line mesh tracing its smoothed centerline, tinted as infrastructure.
/// One-shot at spawn — the track is immutable, so the geometry never changes.
fn attach_track_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    query: Query<(Entity, &RailTrack), Added<RailTrack>>,
) {
    for (entity, track) in &query {
        let mesh = meshes.add(track_line_mesh(track.points()));
        commands.entity(entity).insert((
            Mesh2d(mesh),
            MeshMaterial2d(materials.add(ColorMaterial::from(RAIL_COLOR))),
        ));
    }
}

/// Give each newly-added locomotive a colored sprite sized to its body collider. Mirrors
/// `attach_player_sprite` — visuals derive from the gameplay collider, so they can't drift.
fn attach_locomotive_sprite(
    mut commands: Commands,
    query: Query<(Entity, &Collider), Added<Locomotive>>,
) {
    for (entity, collider) in &query {
        commands.entity(entity).insert(Sprite {
            color: LOCO_COLOR,
            custom_size: Some(collider.render_size()),
            ..default()
        });
    }
}
