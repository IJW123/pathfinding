use bevy::ecs::entity::Entity;
use bevy::math::IVec2;
use bevy::prelude::Message;

#[derive(Message)]
pub struct ChunkLoaded {
    pub coord: IVec2,
    pub entity: Entity,
}

#[derive(Message)]
pub struct ChunkUnloaded {
    pub coord: IVec2,
    pub entity: Entity,
}
