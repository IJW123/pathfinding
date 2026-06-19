use std::collections::HashMap;
use std::error::Error;
use std::fmt::{self, Display, Formatter};

use bevy::prelude::*;

use hitboxes_rapier::components::Collider;
use hitboxes_rapier::shape::DegenerateHullError;

use crate::components::SpriteId;
use crate::manifest::RawSpriteDef;
use crate::scale::collider_for;

/// In-memory baked sprite data. Same fields as [`RawSpriteDef`] but with the hull mapped to `Vec2`
/// for direct use by [`crate::scale`].
pub struct SpriteDef {
    /// Texture path relative to bevy's `assets/` root, for the render side's `AssetServer`.
    pub image_path: String,
    /// Pixel aspect (width / height). Drives the sprite's non-longest dimension.
    pub aspect: f32,
    /// Normalized silhouette hull: longest image side = 1.0, image-center origin, y-up.
    pub hull: Vec<Vec2>,
}

impl From<RawSpriteDef> for SpriteDef {
    fn from(raw: RawSpriteDef) -> Self {
        Self {
            image_path: raw.image_path,
            aspect: raw.aspect,
            hull: raw.hull.into_iter().map(|(x, y)| Vec2::new(x, y)).collect(),
        }
    }
}

/// All baked sprites, keyed by id. Loaded once at startup from the manifest; empty if no manifest
/// exists yet (see [`crate::loader`]).
#[derive(Resource, Default)]
pub struct SpriteCatalog {
    pub defs: HashMap<SpriteId, SpriteDef>,
}

impl SpriteCatalog {
    #[must_use]
    pub fn get(&self, id: &SpriteId) -> Option<&SpriteDef> {
        self.defs.get(id)
    }

    /// Convex collider for sprite `id`, scaled to `world_size`. The spawn-side entry point.
    ///
    /// # Errors
    /// [`SpriteColliderError::Unknown`] if `id` isn't in the catalog (typo, or the manifest wasn't
    /// baked / loaded); [`SpriteColliderError::Degenerate`] if its hull spans no area.
    pub fn collider_for(
        &self,
        id: &SpriteId,
        world_size: f32,
    ) -> Result<Collider, SpriteColliderError> {
        let def = self
            .get(id)
            .ok_or_else(|| SpriteColliderError::Unknown(id.clone()))?;
        collider_for(def, world_size).map_err(SpriteColliderError::Degenerate)
    }
}

/// Why a collider couldn't be built for a sprite id.
#[derive(Debug)]
pub enum SpriteColliderError {
    Unknown(SpriteId),
    Degenerate(DegenerateHullError),
}

impl Display for SpriteColliderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unknown(id) => write!(f, "no baked sprite '{}' in catalog", id.as_str()),
            Self::Degenerate(err) => write!(f, "{err}"),
        }
    }
}

impl Error for SpriteColliderError {}
