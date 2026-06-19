use bevy::prelude::*;

use crate::catalog::SpriteCatalog;
use crate::loader::load_catalog;

/// Owns the sprite catalog: inserts it and loads the baked manifest in `PreStartup`, so it's ready
/// before any `Startup` spawn reads it. World-logic only — no rendering, no image loading.
pub struct SpritesPlugin;

impl Plugin for SpritesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpriteCatalog>()
            .add_systems(PreStartup, load_catalog);
    }
}
