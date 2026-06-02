use bevy::prelude::*;

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
    camera: Single<(&Transform, &Projection), With<Camera2d>>,
) {
    let (transform, projection) = *camera;
    let cam_pos = transform.translation.truncate();
    let Projection::Orthographic(ortho) = projection else {
        return;
    };
    let desired = desired_chunks(cam_pos, ortho.area.size());

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
