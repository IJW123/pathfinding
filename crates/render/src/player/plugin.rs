use bevy::app::{App, Plugin, Update};
use bevy::prelude::*;

use hitboxes::components::Collider;
use player::components::Player;

use crate::player::constants::PLAYER_SPRITE_COLOR;

pub struct PlayerRenderPlugin;

impl Plugin for PlayerRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, attach_player_sprite);
    }
}

fn attach_player_sprite(mut commands: Commands, query: Query<(Entity, &Collider), Added<Player>>) {
    for (entity, collider) in &query {
        commands.entity(entity).insert(Sprite {
            color: PLAYER_SPRITE_COLOR,
            custom_size: Some(collider.render_size()),
            ..default()
        });
    }
}
