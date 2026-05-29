use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use collision::components::{Collider, Solid, Static};
use crate::components::Wall;
use crate::constants::WALL_THICKNESS;

pub fn spawn_bounds(mut commands: Commands, window: Single<&Window, With<PrimaryWindow>>) {
    let half_w = window.width() / 2.0;
    let half_h = window.height() / 2.0;
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
