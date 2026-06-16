use bevy::prelude::*;

use hitboxes_rapier::components::{Collider, Static};

use crate::components::{Inventory, Storage};

/// A storage building: a square, immovable [`Storage`] holding `inventory`. `Solid` is supplied by
/// [`Storage`]'s required components; `Static` is added here because storage is fixed in place (same
/// per-instance distinction `obstacle`'s `static_obstacle` makes). The square is an OBB collider —
/// the renderer derives the mesh from it.
#[must_use]
pub fn storage_building(
    transform: Transform,
    half_extents: Vec2,
    inventory: Inventory,
) -> impl Bundle {
    (
        transform,
        Storage,
        Collider::obb(half_extents),
        inventory,
        Static,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commodity::Commodity;
    use hitboxes_rapier::components::Solid;

    #[test]
    fn storage_building_is_solid_static_with_inventory() {
        let mut world = World::new();
        let inv = Inventory::from_stock([(Commodity::Grain, 42)]);
        let e = world
            .spawn(storage_building(
                Transform::IDENTITY,
                Vec2::splat(30.0),
                inv,
            ))
            .id();

        assert!(world.get::<Storage>(e).is_some());
        assert!(world.get::<Solid>(e).is_some(), "Solid from #[require]");
        assert!(world.get::<Static>(e).is_some(), "storage is immovable");
        assert!(world.get::<Collider>(e).is_some());
        assert_eq!(world.get::<Inventory>(e).copied(), Some(inv));
    }
}
