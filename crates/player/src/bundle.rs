use bevy::prelude::*;

use hitboxes_rapier::components::Collider;
use motion::components::PrevPosition;

use crate::components::Player;
use crate::constants::{PLAYER_SIZE, PLAYER_Z};

/// The player avatar at `spawn`. `Solid` and `MeasuredVelocity` come from [`Player`]'s required
/// components; `PrevPosition` is seeded to `spawn` to avoid a first-frame velocity spike.
#[must_use]
pub fn player(spawn: Vec2) -> impl Bundle {
    (
        Transform::from_xyz(spawn.x, spawn.y, PLAYER_Z),
        Player,
        Collider::obb(Vec2::splat(PLAYER_SIZE / 2.0)),
        PrevPosition(spawn),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use hitboxes_rapier::components::Solid;
    use motion::components::MeasuredVelocity;

    #[test]
    fn player_has_required_and_seeded_components() {
        let spawn = Vec2::new(3.0, -4.0);
        let mut world = World::new();
        let e = world.spawn(player(spawn)).id();
        assert!(world.get::<Player>(e).is_some());
        assert!(world.get::<Solid>(e).is_some(), "Solid from #[require]");
        assert!(
            world.get::<MeasuredVelocity>(e).is_some(),
            "MeasuredVelocity from #[require]"
        );
        assert!(world.get::<Collider>(e).is_some());
        assert_eq!(
            world.get::<PrevPosition>(e).expect("seeded").0,
            spawn,
            "PrevPosition seeded to spawn"
        );
    }
}
