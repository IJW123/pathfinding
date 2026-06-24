use bevy::color::Color;
use bevy::color::palettes::css::{DARK_GRAY, ORANGE_RED};

/// Rail line color. A muted gray so the track reads as infrastructure, not a unit.
pub const RAIL_COLOR: Color = Color::Srgba(DARK_GRAY);

/// Locomotive body color. A warm accent so the single loco stands out against units/obstacles.
pub const LOCO_COLOR: Color = Color::Srgba(ORANGE_RED);
