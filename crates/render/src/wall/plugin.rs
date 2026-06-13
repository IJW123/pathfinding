use bevy::app::{App, Plugin, Update};
use bevy::prelude::*;

use hitboxes_rapier::components::Collider;
use world::components::Wall;

use crate::wall::constants::WALL_SPRITE_COLOR;

pub struct WallRenderPlugin;

impl Plugin for WallRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, attach_wall_sprite);
    }
}

fn attach_wall_sprite(mut commands: Commands, query: Query<(Entity, &Collider), Added<Wall>>) {
    for (entity, collider) in &query {
        commands.entity(entity).insert(Sprite {
            color: WALL_SPRITE_COLOR,
            custom_size: Some(collider.render_size()),
            ..default()
        });
    }
}
