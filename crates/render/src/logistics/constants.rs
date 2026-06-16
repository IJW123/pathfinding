use bevy::color::Color;

/// Storage buildings render distinct from obstacles — a warm wood/grain tone.
pub const STORAGE_COLOR: Color = Color::srgb(0.6, 0.45, 0.25);

/// Dock zone fill: the storage tone at low alpha, so the load/unload radius reads as a faint disc.
pub const DOCK_ZONE_COLOR: Color = Color::srgba(0.6, 0.45, 0.25, 0.12);

/// Local Z offset of the zone disc relative to its storage parent — negative so it sits *under* the
/// building (parent 0.5 → world 0.2, still above contour lines at 0.1).
pub const DOCK_ZONE_Z_OFFSET: f32 = -0.3;
