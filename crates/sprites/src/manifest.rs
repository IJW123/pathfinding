//! Serde-facing mirror of `spritebake`'s on-disk format. Kept separate from the in-memory
//! [`crate::catalog`] types so the file format and runtime representation can evolve independently,
//! and so we deserialize hull points as plain `(f32, f32)` tuples — no glam `serde` feature needed.

use std::collections::BTreeMap;

use serde::Deserialize;

/// One sprite as written by the bake tool. `hull` is normalized: longest image side = 1.0,
/// image-center origin, y-up.
#[derive(Deserialize)]
pub struct RawSpriteDef {
    pub image_path: String,
    pub aspect: f32,
    pub hull: Vec<(f32, f32)>,
}

/// Top-level manifest: sprite id -> def.
#[derive(Deserialize)]
pub struct RawManifest {
    pub sprites: BTreeMap<String, RawSpriteDef>,
}
