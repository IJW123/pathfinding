//! Storage building composition: a square `Storage` plus everything the level wires onto it —
//! capacity, a dock zone, the warehouse sprite, selection, and a velocity readout.

use bevy::prelude::*;

use logistics::bundle::storage_building;
use logistics::cargo_handling::components::DockZone;
use logistics::components::{Capacity, Inventory};
use logistics::constants::STORAGE_Z;
use motion::components::{MeasuredVelocity, PrevPosition};
use selection::components::Selectable;
use sprites::components::{SpriteId, SpriteRef};

use crate::objects::spec::StorageSpec;

/// A storage building from `spec`. Returns the complete entity: the `storage_building` bundle plus
/// the level's composition — capacity capped by volume only, a circular dock zone a carrier must
/// enter, the warehouse sprite, selection, and a velocity readout for when this is the controlled
/// entity.
///
/// `STORAGE_Z` (per-kind render ordering) is applied here; the spec carries `pos` only. `Storage`
/// only requires `Solid`, so `MeasuredVelocity`/`PrevPosition` are added by hand (not free via
/// `#[require]` the way they are on `Player`); `PrevPosition` is seeded to spawn so the first
/// measured tick isn't a spike. The sprite id is a single literal, not authored data, so it stays
/// hardcoded here.
#[must_use]
pub fn storage(spec: &StorageSpec) -> impl Bundle {
    let transform = Transform::from_xyz(spec.pos.x, spec.pos.y, STORAGE_Z);
    let capacity = Capacity {
        max_weight: None,
        max_volume: Some(spec.max_volume),
    };
    let (inventory, overflow) = Inventory::from_stock_capped(&capacity, spec.stock.iter().copied());
    for (commodity, dropped) in overflow {
        warn!(?commodity, dropped, ?spec.pos, "storage seed exceeds volume cap; clamped");
    }
    (
        storage_building(transform, Vec2::splat(spec.half_extent), inventory),
        capacity,
        DockZone {
            radius: spec.dock_radius,
        },
        // The warehouse sprite skins the existing square collider (full side = 2 * half-extent); the
        // OBB collider is unchanged.
        SpriteRef {
            id: SpriteId::new("warehouse"),
            world_size: spec.half_extent * 2.0,
        },
        Selectable,
        MeasuredVelocity::default(),
        PrevPosition(spec.pos),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use logistics::commodity::Commodity;

    #[test]
    fn storage_has_full_composition_and_seeded_prev_position() {
        let spec = StorageSpec {
            pos: Vec2::new(-250.0, 200.0),
            half_extent: 50.0,
            max_volume: 20.0,
            dock_radius: 120.0,
            stock: vec![(Commodity::Grain, 100)],
        };
        let mut world = World::new();
        let e = world.spawn(storage(&spec)).id();

        assert!(world.get::<Capacity>(e).is_some());
        assert!(world.get::<DockZone>(e).is_some());
        assert!(world.get::<SpriteRef>(e).is_some());
        assert!(world.get::<Selectable>(e).is_some());
        assert!(world.get::<MeasuredVelocity>(e).is_some());
        assert_eq!(
            world.get::<PrevPosition>(e).expect("seeded").0,
            spec.pos,
            "PrevPosition seeded to spawn"
        );
    }

    #[test]
    fn over_cap_stock_is_clamped_to_volume() {
        // 20 m³ holds ~615 grain (0.0325 m³/unit); 1000 overshoots → seed must clamp at-or-under cap.
        let spec = StorageSpec {
            pos: Vec2::ZERO,
            half_extent: 50.0,
            max_volume: 20.0,
            dock_radius: 120.0,
            stock: vec![(Commodity::Grain, 1000)],
        };
        let mut world = World::new();
        let e = world.spawn(storage(&spec)).id();
        let inv = world.get::<Inventory>(e).expect("seeded inventory");

        assert!(
            inv.total_volume() <= spec.max_volume + f32::EPSILON,
            "seed clamped to volume cap, got {} m³",
            inv.total_volume()
        );
        assert!(
            inv.amount(Commodity::Grain) > 0,
            "clamp leaves what fits, not nothing"
        );
    }
}
