use std::collections::HashMap;

use bevy::math::IVec2;
use bevy::prelude::{ColorMaterial, Handle, Mesh, Resource};

/// Per-chunk-coord cache of contour render assets.
///
/// The map is fixed and finite and contours derive from the immutable `HeightFn`,
/// so a chunk coord's geometry never changes — compute it once, reattach forever.
/// Holding a **strong** `Handle<Mesh>` here keeps the asset alive after the chunk
/// entity despawns on unload (Bevy ref-counts assets by strong handle).
///
/// # Invalidation
///
/// Assumes `HeightFn`, `ContourLevels`, and `ContourStyle` are immutable (they are
/// `init_resource` defaults, never mutated). If any becomes runtime-mutable, clear
/// this cache when it changes.
#[derive(Resource, Default)]
pub struct ContourCache {
    pub meshes: HashMap<IVec2, Handle<Mesh>>,
    /// Shared white material backing every chunk, created lazily on first use.
    pub material: Option<Handle<ColorMaterial>>,
}
