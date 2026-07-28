use bevy::app::{App, Plugin, Update};
use bevy::prelude::*;
use bevy::sprite_render::AlphaMode2d;

use range::components::RangeRing;

use crate::range::constants::{RANGE_DISC_COLOR, RANGE_DISC_Z};

pub struct RangeRingRenderPlugin;

impl Plugin for RangeRingRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, attach_range_disc);
    }
}

/// Give each newly-added [`RangeRing`] a faint translucent disc showing its radius. A child of the
/// ranged entity so it tracks it; its local Z is offset against the parent's own Z so the disc lands
/// at a fixed world layer ([`RANGE_DISC_Z`]) whatever layer the holder sits on. One general system —
/// stations, and any future ranged thing, opt in just by carrying `RangeRing`.
fn attach_range_disc(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    query: Query<(Entity, &RangeRing, &Transform), Added<RangeRing>>,
) {
    for (entity, ring, transform) in &query {
        let mesh = meshes.add(Circle::new(ring.radius));
        let material = materials.add(ColorMaterial {
            color: RANGE_DISC_COLOR,
            alpha_mode: AlphaMode2d::Blend,
            ..default()
        });
        commands.entity(entity).with_child((
            Mesh2d(mesh),
            MeshMaterial2d(material),
            Transform::from_xyz(0.0, 0.0, RANGE_DISC_Z - transform.translation.z),
        ));
    }
}
