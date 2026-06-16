use bevy::prelude::*;

use crate::commodity::{Commodity, CommodityChange};
use crate::components::{Inventory, Storage};
use crate::message::CommodityTransfer;

/// How much one key press moves.
const STEP: u32 = 10;

/// Debug-only: drive the storage mutation API from the keyboard so the message flow is observable.
/// Digit 1–4 deposit grain/coal/lumber/iron-ore; 5–8 withdraw the same. Targets every [`Storage`].
pub fn debug_storage_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    storages: Query<Entity, With<Storage>>,
    mut transfers: MessageWriter<CommodityTransfer>,
) {
    let bindings = [
        (
            KeyCode::Digit1,
            Commodity::Grain,
            CommodityChange::Deposit(STEP),
        ),
        (
            KeyCode::Digit2,
            Commodity::Coal,
            CommodityChange::Deposit(STEP),
        ),
        (
            KeyCode::Digit3,
            Commodity::Lumber,
            CommodityChange::Deposit(STEP),
        ),
        (
            KeyCode::Digit4,
            Commodity::IronOre,
            CommodityChange::Deposit(STEP),
        ),
        (
            KeyCode::Digit5,
            Commodity::Grain,
            CommodityChange::Withdraw(STEP),
        ),
        (
            KeyCode::Digit6,
            Commodity::Coal,
            CommodityChange::Withdraw(STEP),
        ),
        (
            KeyCode::Digit7,
            Commodity::Lumber,
            CommodityChange::Withdraw(STEP),
        ),
        (
            KeyCode::Digit8,
            Commodity::IronOre,
            CommodityChange::Withdraw(STEP),
        ),
    ];

    for (key, commodity, change) in bindings {
        if keyboard.just_pressed(key) {
            for target in &storages {
                transfers.write(CommodityTransfer {
                    target,
                    commodity,
                    change,
                });
            }
        }
    }
}

/// Debug-only: log a storage's stock whenever it changes, so keyboard-driven transfers are visible.
pub fn log_inventory_changes(changed: Query<(Entity, &Inventory), Changed<Inventory>>) {
    for (entity, inv) in &changed {
        info!(
            "storage {entity}: grain={} coal={} lumber={} iron_ore={}",
            inv.grain, inv.coal, inv.lumber, inv.iron_ore
        );
    }
}
