use bevy::prelude::*;

use hitboxes_rapier::components::{Collider, Static};

use crate::components::{Obstacle, Wall};
use crate::constants::WALL_Z;

/// A static (immovable) obstacle. `Solid` is supplied by [`Obstacle`]'s required components;
/// `Static` is added here because it's the per-instance distinction between static and pushable.
#[must_use]
pub fn static_obstacle(transform: Transform, collider: Collider) -> impl Bundle {
    (transform, Obstacle, collider, Static)
}

/// A pushable (dynamic) obstacle — same as [`static_obstacle`] minus `Static`.
#[must_use]
pub fn pushable_obstacle(transform: Transform, collider: Collider) -> impl Bundle {
    (transform, Obstacle, collider)
}

/// A single boundary wall: a static obstacle with an OBB collider plus the [`Wall`] tag the
/// renderer keys off for the wall color. Centered at `center` with the given half-extents.
#[must_use]
pub fn wall(center: Vec2, half_extents: Vec2) -> impl Bundle {
    (
        static_obstacle(
            Transform::from_xyz(center.x, center.y, WALL_Z),
            Collider::obb(half_extents),
        ),
        Wall,
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
    use hitboxes_rapier::components::Solid;

    fn spawn(bundle: impl Bundle) -> (World, Entity) {
        let mut world = World::new();
        let entity = world.spawn(bundle).id();
        (world, entity)
    }

    #[test]
    fn static_obstacle_is_solid_and_static() {
        let (world, e) = spawn(static_obstacle(Transform::IDENTITY, Collider::circle(1.0)));
        assert!(world.get::<Obstacle>(e).is_some());
        assert!(world.get::<Solid>(e).is_some(), "Solid from #[require]");
        assert!(world.get::<Static>(e).is_some());
    }

    #[test]
    fn pushable_obstacle_is_solid_not_static() {
        let (world, e) = spawn(pushable_obstacle(
            Transform::IDENTITY,
            Collider::circle(1.0),
        ));
        assert!(world.get::<Obstacle>(e).is_some());
        assert!(world.get::<Solid>(e).is_some(), "Solid from #[require]");
        assert!(world.get::<Static>(e).is_none(), "pushables omit Static");
    }

    #[test]
    fn wall_is_obstacle_solid_and_static() {
        let (world, e) = spawn(wall(Vec2::ZERO, Vec2::splat(10.0)));
        assert!(world.get::<Wall>(e).is_some());
        assert!(world.get::<Obstacle>(e).is_some(), "walls are obstacles");
        assert!(world.get::<Solid>(e).is_some(), "Solid from #[require]");
        assert!(world.get::<Static>(e).is_some(), "walls are static");
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
