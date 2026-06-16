use bevy::prelude::*;

use hitboxes_rapier::components::Solid;

use crate::commodity::{Commodity, CommodityChange};

/// Marker for storage buildings. Render keys off it via `Added<Storage>`; the collision pipeline
/// keys off the required `Solid`. `Static` is *not* required — like [`obstacle`]'s `Obstacle`,
/// immovability is a per-instance choice made in the bundle.
#[derive(Component)]
#[require(Solid)]
pub struct Storage;

/// The goods an entity currently holds. General-purpose: a warehouse, a transport truck, a ship —
/// anything that carries commodities gets one (it is *not* tied to [`Storage`]). Unbounded above;
/// every count clamps at 0 below. All mutation routes through [`Inventory::apply`] so the message
/// system and any future caller share one path.
#[derive(Component, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Inventory {
    pub grain: u32,
    pub coal: u32,
    pub lumber: u32,
    pub iron_ore: u32,
}

impl Inventory {
    #[must_use]
    pub fn amount(&self, commodity: Commodity) -> u32 {
        match commodity {
            Commodity::Grain => self.grain,
            Commodity::Coal => self.coal,
            Commodity::Lumber => self.lumber,
            Commodity::IronOre => self.iron_ore,
        }
    }

    fn slot_mut(&mut self, commodity: Commodity) -> &mut u32 {
        match commodity {
            Commodity::Grain => &mut self.grain,
            Commodity::Coal => &mut self.coal,
            Commodity::Lumber => &mut self.lumber,
            Commodity::IronOre => &mut self.iron_ore,
        }
    }

    /// Add `amount` of `commodity` (saturating).
    pub fn add(&mut self, commodity: Commodity, amount: u32) {
        let slot = self.slot_mut(commodity);
        *slot = slot.saturating_add(amount);
    }

    /// Remove up to `amount` of `commodity`, clamping at 0. Returns how much was actually removed.
    pub fn remove(&mut self, commodity: Commodity, amount: u32) -> u32 {
        let slot = self.slot_mut(commodity);
        let removed = (*slot).min(amount);
        *slot -= removed;
        removed
    }

    /// Apply a [`CommodityChange`] to `commodity`.
    pub fn apply(&mut self, commodity: Commodity, change: CommodityChange) {
        match change {
            CommodityChange::Deposit(amount) => self.add(commodity, amount),
            CommodityChange::Withdraw(amount) => {
                self.remove(commodity, amount);
            }
        }
    }

    /// Total weight of everything held (kg) — the sum over commodities of count × unit weight.
    #[must_use]
    pub fn total_weight(&self) -> f32 {
        Commodity::ALL
            .into_iter()
            .map(|c| self.amount(c) as f32 * c.unit_weight())
            .sum()
    }

    /// Total volume of everything held (m³) — the sum over commodities of count × unit volume.
    #[must_use]
    pub fn total_volume(&self) -> f32 {
        Commodity::ALL
            .into_iter()
            .map(|c| self.amount(c) as f32 * c.unit_volume())
            .sum()
    }
}

/// How much an entity may hold, per axis. `None` means unbounded on that axis: a storage building
/// caps volume only, a carrier caps both. Density decides which cap binds first — a dense good
/// fills the weight cap before the volume cap, a bulky one the reverse. Holders without a `Capacity`
/// are unbounded (preserves the original deposit-anything behaviour).
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Capacity {
    pub max_weight: Option<f32>,
    pub max_volume: Option<f32>,
}

impl Capacity {
    /// Units of `commodity` depositable into `inv` without breaching either cap. Floors on the
    /// binding axis; returns `requested` when both axes are unbounded.
    #[must_use]
    pub fn grantable(&self, inv: &Inventory, commodity: Commodity, requested: u32) -> u32 {
        let headroom = |cap: Option<f32>, used: f32, per_unit: f32| match cap {
            Some(max) if per_unit > 0.0 => ((max - used) / per_unit).floor().max(0.0) as u32,
            _ => requested,
        };
        requested
            .min(headroom(
                self.max_weight,
                inv.total_weight(),
                commodity.unit_weight(),
            ))
            .min(headroom(
                self.max_volume,
                inv.total_volume(),
                commodity.unit_volume(),
            ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_accumulates() {
        let mut inv = Inventory::default();
        inv.add(Commodity::Grain, 10);
        inv.add(Commodity::Grain, 5);
        assert_eq!(inv.amount(Commodity::Grain), 15);
    }

    #[test]
    fn remove_clamps_at_zero_and_reports_actual() {
        let mut inv = Inventory {
            coal: 3,
            ..Inventory::default()
        };
        assert_eq!(inv.remove(Commodity::Coal, 10), 3, "only 3 available");
        assert_eq!(inv.amount(Commodity::Coal), 0);
    }

    #[test]
    fn apply_dispatches_deposit_and_withdraw() {
        let mut inv = Inventory::default();
        inv.apply(Commodity::Lumber, CommodityChange::Deposit(8));
        inv.apply(Commodity::Lumber, CommodityChange::Withdraw(3));
        assert_eq!(inv.amount(Commodity::Lumber), 5);
    }

    #[test]
    fn totals_sum_weight_and_volume_over_stock() {
        let inv = Inventory {
            grain: 4,
            iron_ore: 2,
            ..Inventory::default()
        };
        let expected_weight =
            4.0 * Commodity::Grain.unit_weight() + 2.0 * Commodity::IronOre.unit_weight();
        let expected_volume =
            4.0 * Commodity::Grain.unit_volume() + 2.0 * Commodity::IronOre.unit_volume();
        assert!((inv.total_weight() - expected_weight).abs() < f32::EPSILON);
        assert!((inv.total_volume() - expected_volume).abs() < f32::EPSILON);
    }

    #[test]
    fn grantable_unbounded_passes_request_through() {
        let cap = Capacity::default();
        assert_eq!(
            cap.grantable(&Inventory::default(), Commodity::Grain, 1000),
            1000
        );
    }

    #[test]
    fn grantable_floors_on_weight() {
        // 100 kg cap, grain weighs 25/unit → 4 units fit, ask for 10.
        let cap = Capacity {
            max_weight: Some(100.0),
            max_volume: None,
        };
        assert_eq!(
            cap.grantable(&Inventory::default(), Commodity::Grain, 10),
            4
        );
    }

    #[test]
    fn grantable_floors_on_volume() {
        // 0.1 m³ cap, grain is 25/770 ≈ 0.0325 m³/unit → 3 units fit.
        let cap = Capacity {
            max_weight: None,
            max_volume: Some(0.1),
        };
        assert_eq!(
            cap.grantable(&Inventory::default(), Commodity::Grain, 10),
            3
        );
    }

    #[test]
    fn grantable_zero_when_already_full() {
        let cap = Capacity {
            max_weight: Some(50.0),
            max_volume: None,
        };
        let inv = Inventory {
            grain: 2, // 50 kg, exactly at the cap
            ..Inventory::default()
        };
        assert_eq!(cap.grantable(&inv, Commodity::Grain, 5), 0);
    }
}
