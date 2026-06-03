use bevy::prelude::*;

use crate::components::Wall;
use crate::constants::{MAP_HALF_EXTENT, WALL_THICKNESS};
use collision::components::{Collider, Solid, Static};

pub fn spawn_bounds(mut commands: Commands) {
    let half_w = MAP_HALF_EXTENT;
    let half_h = MAP_HALF_EXTENT;
    let half_t = WALL_THICKNESS / 2.0;

    let walls = [
        (Vec2::new(0.0, half_h - half_t), Vec2::new(half_w, half_t)),
        (Vec2::new(0.0, -half_h + half_t), Vec2::new(half_w, half_t)),
        (Vec2::new(-half_w + half_t, 0.0), Vec2::new(half_t, half_h)),
        (Vec2::new(half_w - half_t, 0.0), Vec2::new(half_t, half_h)),
    ];

    for (pos, half_extents) in walls {
        commands.spawn((
            Transform::from_xyz(pos.x, pos.y, 0.0),
            Wall,
            Collider { half_extents },
            Solid,
            Static,
        ));
    }
}
