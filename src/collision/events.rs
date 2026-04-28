use bevy::prelude::*;

#[derive(Message)]
pub struct CollisionEvent {
    pub a: Entity,
    pub b: Entity,
}
