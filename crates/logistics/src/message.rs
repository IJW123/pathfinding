use bevy::prelude::*;

use crate::commodity::{Commodity, CommodityChange};

/// Request to move goods in or out of an entity's [`crate::components::Inventory`]. Any crate
/// (production, transport, player) writes this; [`crate::systems::apply_commodity_transfers`] applies
/// it. Decoupling the request from the mutation keeps callers ignorant of the inventory layout.
#[derive(Message)]
pub struct CommodityTransfer {
    pub target: Entity,
    pub commodity: Commodity,
    pub change: CommodityChange,
}
