//! Player composition: the avatar doubles as the cargo carrier and is the default-controlled,
//! selectable entity.

use bevy::prelude::*;

use logistics::cargo_handling::components::Carrier;
use logistics::components::{Capacity, Inventory};
use player::bundle::player;
use selection::components::{Selectable, Selected};

use crate::objects::constants::{CARRIER_MAX_VOLUME, CARRIER_MAX_WEIGHT};

/// The player avatar at `spawn`, doubling as the cargo carrier: an empty inventory capped on both
/// weight and volume (so hauling a full building clamps), and selectable + `Selected` so it's the
/// default-controlled entity until the user picks something else.
///
/// `player(spawn)` already ships `PrevPosition(spawn)` and `Player` requires `MeasuredVelocity`, so
/// neither is re-added here.
#[must_use]
pub fn carrier_player(spawn: Vec2) -> impl Bundle {
    (
        player(spawn),
        Inventory::default(),
        Carrier,
        Capacity {
            max_weight: Some(CARRIER_MAX_WEIGHT),
            max_volume: Some(CARRIER_MAX_VOLUME),
        },
        Selectable,
        Selected,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn carrier_player_has_full_composition() {
        let mut world = World::new();
        let e = world.spawn(carrier_player(Vec2::ZERO)).id();

        assert!(world.get::<Inventory>(e).is_some());
        assert!(world.get::<Carrier>(e).is_some());
        assert!(world.get::<Capacity>(e).is_some());
        assert!(world.get::<Selectable>(e).is_some());
        assert!(world.get::<Selected>(e).is_some());
    }
}
