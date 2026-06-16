use bevy::prelude::*;

use crate::commodity::CommodityChange;
use crate::components::{Capacity, Inventory};
use crate::message::CommodityTransfer;

/// Drain [`CommodityTransfer`] messages, applying each to its target's [`Inventory`]. A deposit is
/// clamped to the target's [`Capacity`] (partial fill); a target without a `Capacity` is unbounded.
/// Withdrawals are never capped (caps are upper bounds only). Messages aimed at an entity without an
/// `Inventory` are silently dropped — the sender doesn't get to assume the target carries goods.
pub fn apply_commodity_transfers(
    mut reader: MessageReader<CommodityTransfer>,
    mut holders: Query<(&mut Inventory, Option<&Capacity>)>,
) {
    for transfer in reader.read() {
        if let Ok((mut inventory, capacity)) = holders.get_mut(transfer.target) {
            let change = match transfer.change {
                CommodityChange::Deposit(amount) => {
                    let granted = capacity.map_or(amount, |c| {
                        c.grantable(&inventory, transfer.commodity, amount)
                    });
                    CommodityChange::Deposit(granted)
                }
                withdraw => withdraw,
            };
            inventory.apply(transfer.commodity, change);
        }
    }
}
