use bevy::color::Color;

/// Bright outline tint for the selected entity's highlight ring.
pub const HIGHLIGHT_COLOR: Color = Color::srgb(1.0, 0.9, 0.2);

/// The highlight child mesh is the collider silhouette scaled up slightly, so its edge peeks out
/// from behind the entity as a border ring.
pub const HIGHLIGHT_SCALE: f32 = 1.12;

/// Child-local Z: a touch behind the parent so the larger ring sits under it. Local only — it
/// shifts the highlight relative to its holder, not into other entities' layers.
pub const HIGHLIGHT_Z_OFFSET: f32 = -0.05;
