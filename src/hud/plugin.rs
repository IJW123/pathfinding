use bevy::app::{App, Plugin, Startup, Update};

use crate::hud::systems::{spawn_hud, update_coord_text};

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_hud)
            .add_systems(Update, update_coord_text);
    }
}
