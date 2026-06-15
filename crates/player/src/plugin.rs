use bevy::prelude::*;

use crate::systems::move_player;
use collision_rapier::plugin::CollisionSet;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        // FixedUpdate: `Res<Time>` resolves to the fixed clock, so the per-tick step is
        // bounded (speed ÷ tick rate) and collision can guarantee no tunneling.
        app.add_systems(FixedUpdate, move_player.before(CollisionSet));
    }
}
