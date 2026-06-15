use bevy::prelude::*;

use hitboxes_rapier::components::Solid;

/// Marker for placed obstacle entities. Render keys off it via `Added<Obstacle>`.
///
/// Requires `Solid` (every obstacle collides). `Static` is *not* required — pushable
/// obstacles omit it — so immovability is decided per-instance at spawn.
#[derive(Component)]
#[require(Solid)]
pub struct Obstacle;
