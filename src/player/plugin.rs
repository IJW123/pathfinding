use bevy::prelude::*;

use crate::collision::plugin::CollisionSet;
use crate::player::systems::{move_player, setup_player};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_player)
            .add_systems(Update, move_player.before(CollisionSet));
    }
}
