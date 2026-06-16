use crate::constants::{
    COAL_DENSITY, COAL_UNIT_WEIGHT, GRAIN_DENSITY, GRAIN_UNIT_WEIGHT, IRON_ORE_DENSITY,
    IRON_ORE_UNIT_WEIGHT, LUMBER_DENSITY, LUMBER_UNIT_WEIGHT,
};

/// A tradeable good. The enum is the unit of mutation: add/remove and the
/// [`crate::message::CommodityTransfer`] message are written once over `Commodity`, not per field.
/// Adding a good touches the variant here (plus its [`Commodity::props`] arm and [`Commodity::ALL`])
/// and the matching field + slot mapping in [`crate::components::Inventory`]. Each variant carries
/// physical properties (weight per unit, density) so an inventory can report total weight and volume.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Commodity {
    Grain,
    Coal,
    Lumber,
    IronOre,
}

/// The physical properties of one commodity, grouped so each good's data lives in a single
/// [`Commodity::props`] arm. Adding a new property costs one field here plus one value per commodity
/// arm — not a whole new parallel match. The numbers live in `constants.rs`.
pub struct CommodityProps {
    /// Weight of one unit (kg).
    pub unit_weight: f32,
    /// Density (kg/m³).
    pub density: f32,
}

/// A requested change to one commodity. `Withdraw` clamps at 0 — an inventory is unbounded above but
/// never goes negative.
#[derive(Clone, Copy, Debug)]
pub enum CommodityChange {
    Deposit(u32),
    Withdraw(u32),
}

impl Commodity {
    /// Every commodity, for folding over an [`crate::components::Inventory`].
    pub const ALL: [Commodity; 4] = [
        Commodity::Grain,
        Commodity::Coal,
        Commodity::Lumber,
        Commodity::IronOre,
    ];

    /// Physical properties of this commodity — the single per-commodity record. `constants.rs`
    /// holds the numbers; this match is the only place they're keyed to a variant.
    #[must_use]
    pub const fn props(self) -> CommodityProps {
        match self {
            Commodity::Grain => CommodityProps {
                unit_weight: GRAIN_UNIT_WEIGHT,
                density: GRAIN_DENSITY,
            },
            Commodity::Coal => CommodityProps {
                unit_weight: COAL_UNIT_WEIGHT,
                density: COAL_DENSITY,
            },
            Commodity::Lumber => CommodityProps {
                unit_weight: LUMBER_UNIT_WEIGHT,
                density: LUMBER_DENSITY,
            },
            Commodity::IronOre => CommodityProps {
                unit_weight: IRON_ORE_UNIT_WEIGHT,
                density: IRON_ORE_DENSITY,
            },
        }
    }

    /// Weight of one unit (kg).
    #[must_use]
    pub fn unit_weight(self) -> f32 {
        self.props().unit_weight
    }

    /// Density (kg/m³).
    #[must_use]
    pub fn density(self) -> f32 {
        self.props().density
    }

    /// Volume of one unit (m³), derived from weight and density.
    #[must_use]
    pub fn unit_volume(self) -> f32 {
        self.unit_weight() / self.density()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_volume_is_weight_over_density() {
        for commodity in Commodity::ALL {
            let expected = commodity.unit_weight() / commodity.density();
            assert!((commodity.unit_volume() - expected).abs() < f32::EPSILON);
        }
    }
}
