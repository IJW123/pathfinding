use bevy::color::Color;

/// Platform bar extent *along* the track (world units) — roughly a train-length, so a stopped loco
/// reads as sitting at the platform.
pub const PLATFORM_LENGTH: f32 = 120.0;

/// Platform bar extent *across* the track (world units): a thick line, clearly heftier than the 1px
/// rail it sits on.
pub const PLATFORM_WIDTH: f32 = 14.0;

/// Platform color: a pale concrete gray, distinct from the dark rail line and the warm storage.
pub const STATION_COLOR: Color = Color::srgb(0.62, 0.64, 0.68);
