use bevy::prelude::*;

use crate::spawn::spawn_level;
use crate::terrain::level_terrain;

/// The one place objects enter the world. Content only — wiring (schedule ordering,
/// `configure_sets`) stays in `app`. Owns the terrain recipe too: inserts `TerrainConfig` for
/// `world`'s elevation engine to consume.
pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(level_terrain())
            .add_systems(Startup, spawn_level);
    }
}
