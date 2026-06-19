//! Player composition: the avatar doubles as the cargo carrier and is the default-controlled,
//! selectable entity.

use bevy::prelude::*;

use logistics::cargo_handling::components::Carrier;
use logistics::components::{Capacity, Inventory};
use player::bundle::player;
use selection::components::{Selectable, Selected};

use crate::objects::spec::CarrierSpec;

/// The player avatar from `spec`, doubling as the cargo carrier: an empty inventory capped on both
/// weight and volume (so hauling a full building clamps), and selectable + `Selected` so it's the
/// default-controlled entity until the user picks something else.
///
/// `player(spawn)` already ships `PrevPosition(spawn)` and applies the player Z, and `Player`
/// requires `MeasuredVelocity`, so neither is re-added here.
#[must_use]
pub fn carrier_player(spec: &CarrierSpec) -> impl Bundle {
    (
        player(spec.spawn),
        Inventory::default(),
        Carrier,
        Capacity {
            max_weight: Some(spec.max_weight),
            max_volume: Some(spec.max_volume),
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
        let spec = CarrierSpec {
            spawn: Vec2::ZERO,
            max_weight: 2000.0,
            max_volume: 3.0,
        };
        let mut world = World::new();
        let e = world.spawn(carrier_player(&spec)).id();

        assert!(world.get::<Inventory>(e).is_some());
        assert!(world.get::<Carrier>(e).is_some());
        assert!(world.get::<Capacity>(e).is_some());
        assert!(world.get::<Selectable>(e).is_some());
        assert!(world.get::<Selected>(e).is_some());
    }
}
