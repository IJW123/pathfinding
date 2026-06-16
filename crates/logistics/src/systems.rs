use bevy::prelude::*;

use crate::components::Inventory;
use crate::message::CommodityTransfer;

/// Drain [`CommodityTransfer`] messages, applying each to its target's [`Inventory`]. Messages aimed
/// at an entity without an `Inventory` are silently dropped — the sender doesn't get to assume the
/// target carries goods.
pub fn apply_commodity_transfers(
    mut reader: MessageReader<CommodityTransfer>,
    mut inventories: Query<&mut Inventory>,
) {
    for transfer in reader.read() {
        if let Ok(mut inventory) = inventories.get_mut(transfer.target) {
            inventory.apply(transfer.commodity, transfer.change);
        }
    }
}
