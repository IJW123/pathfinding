use bevy::color::Color;

/// Maximum on-screen length (logical px) the scale bar may occupy. The labelled
/// distance is the largest "nice" value whose bar fits within this.
pub const TARGET_BAR_PX: f32 = 160.0;

/// Thickness of the scale bar, in logical px.
pub const SCALE_BAR_HEIGHT_PX: f32 = 4.0;

/// Padding from the window edges for HUD elements, in logical px.
pub const HUD_MARGIN_PX: f32 = 8.0;

pub const HUD_FONT_SIZE: f32 = 18.0;

pub const HUD_TEXT_COLOR: Color = Color::srgb(0.95, 0.95, 0.95);
pub const SCALE_BAR_COLOR: Color = Color::srgb(0.95, 0.95, 0.95);
