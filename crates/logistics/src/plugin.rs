use bevy::prelude::*;

use crate::message::CommodityTransfer;
use crate::systems::apply_commodity_transfers;

/// Wires the logistics domain: registers the [`CommodityTransfer`] message and the system that
/// applies it. Content only — no spawning (that's `level`) and no rendering (that's `render`). In
/// debug builds it also adds the [`crate::debug`] keyboard driver so the message flow is observable;
/// that scaffolding compiles out of release.
pub struct LogisticsPlugin;

impl Plugin for LogisticsPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<CommodityTransfer>()
            .add_systems(Update, apply_commodity_transfers);

        // Dev-only keyboard driver. `debug_storage_input` reads `ButtonInput`, so it only runs when
        // an `InputPlugin` has supplied that resource — keeps the plugin usable in headless tests.
        #[cfg(debug_assertions)]
        app.add_systems(
            Update,
            (
                crate::debug::debug_storage_input.run_if(resource_exists::<ButtonInput<KeyCode>>),
                crate::debug::log_inventory_changes,
            ),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bundle::storage_building;
    use crate::commodity::{Commodity, CommodityChange};
    use crate::components::Inventory;

    #[test]
    fn message_mutates_target_inventory() {
        let mut app = App::new();
        app.add_plugins(LogisticsPlugin);

        let store = app
            .world_mut()
            .spawn(storage_building(
                Transform::IDENTITY,
                Vec2::splat(20.0),
                Inventory::default(),
            ))
            .id();

        app.world_mut().write_message(CommodityTransfer {
            target: store,
            commodity: Commodity::IronOre,
            change: CommodityChange::Deposit(7),
        });
        app.update();

        assert_eq!(
            app.world().get::<Inventory>(store).map(|i| i.iron_ore),
            Some(7)
        );
    }
}
