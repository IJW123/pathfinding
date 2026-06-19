//! On-disk manifest format. Hull points are plain `(f32, f32)` tuples so this stays free of any
//! math/bevy dependency; the runtime `sprites` crate mirrors this shape and maps the tuples into
//! `Vec2` on load.

use std::collections::BTreeMap;

use serde::Serialize;

/// One sprite's baked data: where its texture lives, its pixel aspect (w / h), and its normalized
/// silhouette hull (longest image side = 1.0, image-center origin, y-up).
#[derive(Serialize)]
pub struct RawSpriteDef {
    pub image_path: String,
    pub aspect: f32,
    pub hull: Vec<(f32, f32)>,
}

/// Top-level manifest: sprite id (file stem) -> def. `BTreeMap` for deterministic output ordering.
#[derive(Serialize)]
pub struct Manifest {
    pub sprites: BTreeMap<String, RawSpriteDef>,
}
