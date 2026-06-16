use bevy::prelude::*;

/// Move *all* commodities from `source` to `dest`, as much as the dest's capacity allows; whatever
/// doesn't fit stays in `source`. Written by docking interactions, applied by
/// [`crate::cargo_handling::systems::apply_cargo_hauls`].
#[derive(Message)]
pub struct CargoHaul {
    pub source: Entity,
    pub dest: Entity,
}
