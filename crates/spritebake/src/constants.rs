//! Bake-tool knobs. Paths are relative to the workspace root (where `cargo run` runs).

/// Minimum alpha for a pixel to count as part of the silhouette. Pixels below this are treated as
/// fully transparent and don't contribute to the hull.
pub const ALPHA_OPAQUE: u8 = 128;

/// Directory scanned for sprite PNGs; each `*.png` becomes one manifest entry keyed by file stem.
pub const SPRITES_DIR: &str = "assets/sprites";

/// Path prefix recorded in the manifest for each image, relative to bevy's `assets/` root (so the
/// runtime `AssetServer` can load it directly).
pub const ASSET_PATH_PREFIX: &str = "sprites";

/// Where the baked manifest is written.
pub const MANIFEST_PATH: &str = "assets/sprite_manifest.ron";
