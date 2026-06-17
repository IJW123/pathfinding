use bevy::prelude::*;

use collision_rapier::plugin::CollisionSet;

use crate::systems::{move_selected, select_on_click};

pub struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        // Picking reads the cursor in `Update`; movement runs in `FixedUpdate` before collision
        // so the fixed clock bounds the per-tick step (mirrors the old `move_player`).
        app.add_systems(Update, select_on_click)
            .add_systems(FixedUpdate, move_selected.before(CollisionSet));
    }
}
