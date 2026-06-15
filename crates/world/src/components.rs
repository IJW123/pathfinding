use bevy::prelude::*;

use hitboxes_rapier::components::{Solid, Static};

/// Boundary wall marker. Walls are always solid and immovable, so both flags come free.
#[derive(Component)]
#[require(Solid, Static)]
pub struct Wall;
