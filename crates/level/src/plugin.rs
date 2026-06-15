use bevy::prelude::*;

use crate::spawn::spawn_level;

/// The one place objects enter the world. Content only — wiring (schedule ordering,
/// `configure_sets`) stays in `app`.
pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_level);
    }
}
