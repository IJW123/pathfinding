use bevy::app::{App, Plugin, Update};
use bevy::ecs::schedule::SystemSet;
use bevy::prelude::IntoScheduleConfigs;

use crate::elevation::chunk_events::{ChunkLoaded, ChunkUnloaded};
use crate::elevation::chunk_lifecycle::update_loaded_chunks;
use crate::elevation::height_fn::HeightFn;
use crate::elevation::loaded_chunks::LoadedChunks;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ElevationLifecycleSet;

pub struct ElevationPlugin;

impl Plugin for ElevationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HeightFn>()
            .init_resource::<LoadedChunks>()
            .add_message::<ChunkLoaded>()
            .add_message::<ChunkUnloaded>()
            .add_systems(Update, update_loaded_chunks.in_set(ElevationLifecycleSet));
    }
}
