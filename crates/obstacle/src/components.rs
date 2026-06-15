use bevy::prelude::*;

use hitboxes_rapier::components::Solid;

/// Marker for placed obstacle entities. Render keys off it via `Added<Obstacle>`.
///
/// Requires `Solid` (every obstacle collides). `Static` is *not* required — pushable
/// obstacles omit it — so immovability is decided per-instance at spawn.
#[derive(Component)]
#[require(Solid)]
pub struct Obstacle;

/// Boundary-wall sub-kind: an [`Obstacle`] tag the renderer keys off for the wall color. Walls are
/// built via [`crate::bundle::wall`], which supplies `Obstacle`/`Solid`/`Static` — so this is a
/// plain tag with no required components of its own.
#[derive(Component)]
pub struct Wall;
