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
/// system and any future caller share one path. Counts live in an array indexed by
/// `Commodity as usize`, so adding a good costs nothing here — only the enum side grows.
#[derive(Component, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Inventory {
    counts: [u32; Commodity::COUNT],
}

impl Inventory {
    /// Build an inventory from `(commodity, count)` pairs; unmentioned goods start at 0. Repeated
    /// commodities accumulate (saturating), since each pair routes through [`Inventory::add`].
    #[must_use]
    pub fn from_stock(stock: impl IntoIterator<Item = (Commodity, u32)>) -> Self {
        let mut inventory = Self::default();
        for (commodity, amount) in stock {
            inventory.add(commodity, amount);
        }
        inventory
    }

    /// Build an inventory from `(commodity, count)` pairs, clamping each against `cap` as the running
    /// total grows. Returns the inventory plus any rejected overflow `(commodity, dropped)`, so an
    /// over-cap seed is impossible to construct yet authoring mistakes still surface. Capping routes
    /// through [`Capacity::grantable`] — the same path the transfer/haul systems use — so seeding and
    /// depositing honour the caps identically. Like the haul path, the shared weight/volume budget is
    /// consumed in pair order: an earlier commodity eats headroom a later one then sees gone.
    #[must_use]
    pub fn from_stock_capped(
        cap: &Capacity,
        stock: impl IntoIterator<Item = (Commodity, u32)>,
    ) -> (Self, Vec<(Commodity, u32)>) {
        let mut inventory = Self::default();
        let mut overflow = Vec::new();
        for (commodity, amount) in stock {
            let granted = cap.grantable(&inventory, commodity, amount);
            inventory.add(commodity, granted);
            if granted < amount {
                overflow.push((commodity, amount - granted));
            }
        }
        (inventory, overflow)
    }

    #[must_use]
    pub fn amount(&self, commodity: Commodity) -> u32 {
        self.counts[commodity as usize]
    }

    fn slot_mut(&mut self, commodity: Commodity) -> &mut u32 {
        &mut self.counts[commodity as usize]
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
        let mut inv = Inventory::from_stock([(Commodity::Coal, 3)]);
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
        let inv = Inventory::from_stock([(Commodity::Grain, 4), (Commodity::IronOre, 2)]);
        let expected_weight =
            4.0 * Commodity::Grain.unit_weight() + 2.0 * Commodity::IronOre.unit_weight();
        let expected_volume =
            4.0 * Commodity::Grain.unit_volume() + 2.0 * Commodity::IronOre.unit_volume();
        assert!((inv.total_weight() - expected_weight).abs() < f32::EPSILON);
        assert!((inv.total_volume() - expected_volume).abs() < f32::EPSILON);
    }

    #[test]
    fn from_stock_capped_under_cap_seeds_full_no_overflow() {
        let cap = Capacity {
            max_weight: None,
            max_volume: Some(100.0),
        };
        let (inv, overflow) = Inventory::from_stock_capped(&cap, [(Commodity::Grain, 5)]);
        assert_eq!(inv.amount(Commodity::Grain), 5);
        assert!(overflow.is_empty());
    }

    #[test]
    fn from_stock_capped_clamps_and_reports_overflow() {
        // 100 kg cap, grain weighs 25/unit → 4 fit, ask for 10 → 6 overflow.
        let cap = Capacity {
            max_weight: Some(100.0),
            max_volume: None,
        };
        let (inv, overflow) = Inventory::from_stock_capped(&cap, [(Commodity::Grain, 10)]);
        assert_eq!(inv.amount(Commodity::Grain), 4);
        assert_eq!(overflow, vec![(Commodity::Grain, 6)]);
    }

    #[test]
    fn from_stock_capped_tracks_running_total_across_pairs() {
        // 300 kg cap. Grain (25/unit) seeded first: 10 = 250 kg, no overflow. Coal (30/unit) then
        // sees 50 kg headroom → 1 fits, the other 9 overflow. Proves the clamp follows the running
        // total, not each pair in isolation.
        let cap = Capacity {
            max_weight: Some(300.0),
            max_volume: None,
        };
        let (inv, overflow) =
            Inventory::from_stock_capped(&cap, [(Commodity::Grain, 10), (Commodity::Coal, 10)]);
        assert_eq!(inv.amount(Commodity::Grain), 10);
        assert_eq!(inv.amount(Commodity::Coal), 1);
        assert_eq!(overflow, vec![(Commodity::Coal, 9)]);
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
        let inv = Inventory::from_stock([(Commodity::Grain, 2)]); // 50 kg, exactly at the cap
        assert_eq!(cap.grantable(&inv, Commodity::Grain, 5), 0);
    }
}
