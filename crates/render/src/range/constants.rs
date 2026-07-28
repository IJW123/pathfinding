use bevy::color::Color;

/// Range-disc fill: a faint translucent tint, so a range reads as a soft backdrop disc rather than a
/// solid shape. Neutral and cool to set it apart from buildings.
pub const RANGE_DISC_COLOR: Color = Color::srgba(0.40, 0.62, 0.80, 0.14);

/// World Z the range disc renders at: above contour lines (0.1), below the rail/platforms/buildings —
/// a faint backdrop. The render system offsets against the holder's own Z to land the child here,
/// regardless of which layer the ranged entity sits on.
pub const RANGE_DISC_Z: f32 = 0.15;
