use bevy::prelude::*;

/// A circular range of `radius` world units, centered on the entity. Gameplay data: the radius is set
/// in world logic (a station's catchment, a future tower's reach, …) and the renderer draws it as a
/// disc. Appearance — color, layering — is the renderer's decision and is deliberately not encoded
/// here, so one component serves every range type and the visual stays a render concern.
#[derive(Component, Debug, Clone, Copy)]
pub struct RangeRing {
    pub radius: f32,
}
