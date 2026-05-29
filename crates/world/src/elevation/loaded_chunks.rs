use std::collections::HashMap;

use bevy::ecs::entity::Entity;
use bevy::math::IVec2;
use bevy::prelude::Resource;

#[derive(Resource, Default)]
pub struct LoadedChunks(pub HashMap<IVec2, Entity>);
