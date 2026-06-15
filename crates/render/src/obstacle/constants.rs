use bevy::color::Color;

pub const OBSTACLE_STATIC_COLOR: Color = Color::srgb(0.55, 0.55, 0.62);
pub const OBSTACLE_DYNAMIC_COLOR: Color = Color::srgb(0.85, 0.6, 0.2);

/// Boundary walls render distinct from interior static obstacles.
pub const OBSTACLE_WALL_COLOR: Color = Color::srgb(0.4, 0.4, 0.4);
