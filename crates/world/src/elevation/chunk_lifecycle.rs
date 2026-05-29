use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::elevation::chunk_coord::chunk_origin_world;
use crate::elevation::chunk_events::{ChunkLoaded, ChunkUnloaded};
use crate::elevation::chunk_view::desired_chunks;
use crate::elevation::components::ElevationChunk;
use crate::elevation::loaded_chunks::LoadedChunks;

pub fn update_loaded_chunks(
    mut commands: Commands,
    mut loaded: ResMut<LoadedChunks>,
    mut loaded_evw: MessageWriter<ChunkLoaded>,
    mut unloaded_evw: MessageWriter<ChunkUnloaded>,
    camera: Single<&Transform, With<Camera2d>>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    let cam_pos = camera.translation.truncate();
    let viewport = Vec2::new(window.width(), window.height());
    let desired = desired_chunks(cam_pos, viewport);

    for coord in &desired {
        if loaded.0.contains_key(coord) {
            continue;
        }
        let origin = chunk_origin_world(*coord);
        let entity = commands
            .spawn((
                Transform::from_xyz(origin.x, origin.y, 0.1),
                ElevationChunk,
            ))
            .id();
        loaded.0.insert(*coord, entity);
        loaded_evw.write(ChunkLoaded { coord: *coord, entity });
    }

    let stale: Vec<IVec2> = loaded
        .0
        .keys()
        .filter(|k| !desired.contains(k))
        .copied()
        .collect();
    for coord in stale {
        if let Some(entity) = loaded.0.remove(&coord) {
            unloaded_evw.write(ChunkUnloaded { coord, entity });
            commands.entity(entity).despawn();
        }
    }
}
