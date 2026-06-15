use bevy::prelude::*;

use hitboxes_rapier::components::Collider;

use crate::components::Wall;
use crate::constants::WALL_Z;

/// A single boundary wall centered at `center` with the given half-extents. `Solid` and `Static`
/// come from [`Wall`]'s required components.
#[must_use]
pub fn wall(center: Vec2, half_extents: Vec2) -> impl Bundle {
    (
        Transform::from_xyz(center.x, center.y, WALL_Z),
        Wall,
        Collider::obb(half_extents),
    )
}

/// The four map-boundary walls, with positions and extents derived from the map half-extent and
/// wall thickness — placement is parameterized by map size, not authored per wall.
#[must_use]
pub fn boundary_walls(half_extent: f32, thickness: f32) -> [impl Bundle; 4] {
    let half_t = thickness / 2.0;
    [
        wall(
            Vec2::new(0.0, half_extent - half_t),
            Vec2::new(half_extent, half_t),
        ),
        wall(
            Vec2::new(0.0, -half_extent + half_t),
            Vec2::new(half_extent, half_t),
        ),
        wall(
            Vec2::new(-half_extent + half_t, 0.0),
            Vec2::new(half_t, half_extent),
        ),
        wall(
            Vec2::new(half_extent - half_t, 0.0),
            Vec2::new(half_t, half_extent),
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use hitboxes_rapier::components::{Solid, Static};

    #[test]
    fn wall_is_solid_and_static() {
        let mut world = World::new();
        let e = world.spawn(wall(Vec2::ZERO, Vec2::splat(10.0))).id();
        assert!(world.get::<Wall>(e).is_some());
        assert!(world.get::<Solid>(e).is_some(), "Solid from #[require]");
        assert!(world.get::<Static>(e).is_some(), "Static from #[require]");
    }

    #[test]
    fn boundary_walls_spawns_four() {
        let mut world = World::new();
        let count = boundary_walls(100.0, 20.0)
            .into_iter()
            .map(|bundle| world.spawn(bundle).id())
            .count();
        assert_eq!(count, 4);
    }
}
