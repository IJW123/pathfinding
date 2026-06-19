//! Storage building composition: a square `Storage` plus everything the level wires onto it —
//! capacity, a dock zone, the warehouse sprite, selection, and a velocity readout.

use bevy::prelude::*;

use logistics::bundle::storage_building;
use logistics::cargo_handling::components::DockZone;
use logistics::components::{Capacity, Inventory};
use motion::components::{MeasuredVelocity, PrevPosition};
use selection::components::Selectable;
use sprites::components::{SpriteId, SpriteRef};

use crate::objects::constants::{STORAGE_DOCK_RADIUS, STORAGE_MAX_VOLUME};

/// A storage building at `transform`, square with the given `half_extent`, holding `stock`. Returns
/// the complete entity: the `storage_building` bundle plus the level's composition — capacity capped
/// by volume only, a circular dock zone a carrier must enter, the warehouse sprite, selection, and a
/// velocity readout for when this is the controlled entity.
///
/// `half_extent` is a level-side layout knob (the size), so it's passed in rather than owned here.
/// `Storage` only requires `Solid`, so `MeasuredVelocity`/`PrevPosition` are added by hand (they are
/// not free via `#[require]` the way they are on `Player`); `PrevPosition` is seeded to spawn so the
/// first measured tick isn't a spike.
#[must_use]
pub fn storage(transform: Transform, half_extent: f32, stock: Inventory) -> impl Bundle {
    let spawn = transform.translation.xy();
    (
        storage_building(transform, Vec2::splat(half_extent), stock),
        Capacity {
            max_weight: None,
            max_volume: Some(STORAGE_MAX_VOLUME),
        },
        DockZone {
            radius: STORAGE_DOCK_RADIUS,
        },
        // The warehouse sprite skins the existing square collider (full side = 2 * half-extent); the
        // OBB collider is unchanged.
        SpriteRef {
            id: SpriteId::new("warehouse"),
            world_size: half_extent * 2.0,
        },
        Selectable,
        MeasuredVelocity::default(),
        PrevPosition(spawn),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use logistics::commodity::Commodity;

    #[test]
    fn storage_has_full_composition_and_seeded_prev_position() {
        let spawn = Vec2::new(-250.0, 200.0);
        let mut world = World::new();
        let inv = Inventory::from_stock([(Commodity::Grain, 100)]);
        let e = world
            .spawn(storage(
                Transform::from_xyz(spawn.x, spawn.y, 0.0),
                50.0,
                inv,
            ))
            .id();

        assert!(world.get::<Capacity>(e).is_some());
        assert!(world.get::<DockZone>(e).is_some());
        assert!(world.get::<SpriteRef>(e).is_some());
        assert!(world.get::<Selectable>(e).is_some());
        assert!(world.get::<MeasuredVelocity>(e).is_some());
        assert_eq!(
            world.get::<PrevPosition>(e).expect("seeded").0,
            spawn,
            "PrevPosition seeded to spawn"
        );
    }
}
