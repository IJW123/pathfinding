use bevy::prelude::*;

/// Marker for placed obstacle entities. Render keys off it via `Added<Obstacle>`.
#[derive(Component)]
pub struct Obstacle;
