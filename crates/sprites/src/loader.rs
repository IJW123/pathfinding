use std::fs;

use bevy::prelude::*;

use crate::catalog::{SpriteCatalog, SpriteDef};
use crate::components::SpriteId;
use crate::constants::MANIFEST_PATH;
use crate::manifest::RawManifest;

/// PreStartup: read the baked manifest into the [`SpriteCatalog`]. A missing or unparseable
/// manifest is a `warn!`, not a panic — a fresh checkout before the first bake, or a render-less
/// build, should still run. A later `collider_for` on an absent id surfaces the problem with a
/// clear key.
pub fn load_catalog(mut catalog: ResMut<SpriteCatalog>) {
    let text = match fs::read_to_string(MANIFEST_PATH) {
        Ok(text) => text,
        Err(err) => {
            warn!("no sprite manifest at '{MANIFEST_PATH}' ({err}); run `cargo run -p spritebake`");
            return;
        }
    };

    let raw: RawManifest = match ron::from_str(&text) {
        Ok(raw) => raw,
        Err(err) => {
            warn!("sprite manifest '{MANIFEST_PATH}' is unparseable ({err}); catalog left empty");
            return;
        }
    };

    catalog.defs = raw
        .sprites
        .into_iter()
        .map(|(id, def)| (SpriteId::new(id), SpriteDef::from(def)))
        .collect();
    info!(
        "loaded {} sprite(s) from '{MANIFEST_PATH}'",
        catalog.defs.len()
    );
}
