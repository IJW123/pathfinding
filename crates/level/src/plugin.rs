use bevy::prelude::*;

use crate::objects::loader::load_level_spec;
use crate::objects::spec::LevelSpec;
use crate::spawn::spawn_level;

/// The one place objects enter the world. Content only — wiring (schedule ordering,
/// `configure_sets`) stays in `app`. Loads the authored level at build time (so the derived
/// `TerrainConfig` is present before `world`'s `PreStartup` height-field build reads it) and spawns
/// the objects at `Startup`.
pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        // Load from disk only when a spec wasn't already provided. Production: file read + fail-loud.
        // A caller that pre-inserts a `LevelSpec` (tests, overrides) skips disk — and that same spec
        // drives the terrain below, so overriding the level overrides its terrain too.
        if !app.world().contains_resource::<LevelSpec>() {
            app.insert_resource(load_level_spec());
        }
        // Bind before the next insert so the immutable `world()` borrow ends first.
        let terrain = app.world().resource::<LevelSpec>().terrain_config();
        app.insert_resource(terrain)
            .add_systems(Startup, spawn_level);
    }
}
