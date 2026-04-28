use bevy::prelude::*;

#[derive(Component)]
pub struct Collider {
    pub half_extents: Vec2,
}

#[derive(Component)]
pub struct Solid;

#[derive(Component)]
pub struct Static;
