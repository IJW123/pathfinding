use bevy::prelude::*;

use hitboxes_rapier::components::Solid;
use motion::components::MeasuredVelocity;

/// Player avatar marker. `Solid` and `MeasuredVelocity` come free (both fixed-default).
/// `Collider` and `PrevPosition(spawn)` are parameterized, so the constructor inserts them.
#[derive(Component)]
#[require(Solid, MeasuredVelocity)]
pub struct Player;
