use bevy::prelude::*;

/// Stable key for a baked sprite — the PNG's file stem (e.g. `"warehouse"`). The single string
/// that joins the world-logic side (this crate's hull/collider) to the render side (the texture),
/// with no crate dependency between them.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SpriteId(pub String);

impl SpriteId {
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Declares that an entity is drawn as sprite `id`, scaled so its longest side spans `world_size`
/// world units. Both the collider (world logic) and the texture (render) derive from this — see
/// [`crate::scale`]. Spawn code attaches it; the two sides react independently.
#[derive(Component, Clone)]
pub struct SpriteRef {
    pub id: SpriteId,
    pub world_size: f32,
}
