use bevy::prelude::*;

use hitboxes_rapier::components::{Collider, Static};

use crate::components::Obstacle;

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
}
