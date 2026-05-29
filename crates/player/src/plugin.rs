use bevy::prelude::*;

use collision::plugin::CollisionSet;
use crate::systems::{move_player, setup_player};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_player)
            .add_systems(Update, move_player.before(CollisionSet));
    }
}
