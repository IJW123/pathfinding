//! Build-time load of the authored level from `level.ron` into a [`LevelSpec`].

use std::fs;

use crate::objects::constants::LEVEL_PATH;
use crate::objects::manifest::RawLevelSpec;
use crate::objects::spec::LevelSpec;

/// Read `level.ron` into a [`LevelSpec`]. Called from [`LevelPlugin::build`](crate::plugin) (not a
/// Bevy system) so the spec — and the `TerrainConfig` derived from it — exist before any schedule
/// runs, which `world`'s `PreStartup` height-field build requires. Fail-loud: unlike the sprite
/// catalog (a missing texture degrades), a missing or unparseable level means *no world*, so this
/// panics with a path-naming message. The plugin only calls it when a `LevelSpec` wasn't already
/// inserted, so a caller that pre-inserts one (e.g. tests) skips disk entirely.
#[must_use]
pub fn load_level_spec() -> LevelSpec {
    let text = fs::read_to_string(LEVEL_PATH)
        .unwrap_or_else(|err| panic!("cannot read level file '{LEVEL_PATH}': {err}"));
    let raw: RawLevelSpec = ron::from_str(&text)
        .unwrap_or_else(|err| panic!("level file '{LEVEL_PATH}' is unparseable: {err}"));
    LevelSpec::from(raw)
}
