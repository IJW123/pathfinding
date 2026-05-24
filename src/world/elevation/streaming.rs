use std::collections::HashSet;

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::constants::{CHUNK_VIEW_MARGIN, ELEVATION_CELL, ELEV_CHUNK_CELLS};
use crate::world::elevation::components::ElevationChunk;
use crate::world::elevation::height_fn::HeightFn;
use crate::world::elevation::mesh_build::{build_chunk_mesh, chunk_origin_world};
use crate::world::elevation::resources::LoadedChunks;

pub fn update_visible_chunks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut loaded: ResMut<LoadedChunks>,
    height: Res<HeightFn>,
    camera: Single<&Transform, With<Camera2d>>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    let cam_pos = camera.translation.truncate();
    let half = Vec2::new(window.width(), window.height()) * 0.5 + Vec2::splat(CHUNK_VIEW_MARGIN);
    let min = cam_pos - half;
    let max = cam_pos + half;

    let span = ELEV_CHUNK_CELLS as f32 * ELEVATION_CELL;
    let min_cx = (min.x / span).floor() as i32;
    let max_cx = (max.x / span).floor() as i32;
    let min_cy = (min.y / span).floor() as i32;
    let max_cy = (max.y / span).floor() as i32;

    let mut desired: HashSet<IVec2> = HashSet::new();
    for cy in min_cy..=max_cy {
        for cx in min_cx..=max_cx {
            desired.insert(IVec2::new(cx, cy));
        }
    }

    for coord in &desired {
        if loaded.0.contains_key(coord) {
            continue;
        }
        let mesh = build_chunk_mesh(*coord, &height);
        let mesh_handle = meshes.add(mesh);
        let mat_handle = materials.add(ColorMaterial::from(Color::WHITE));
        let origin = chunk_origin_world(*coord);
        let entity = commands
            .spawn((
                Mesh2d(mesh_handle),
                MeshMaterial2d(mat_handle),
                Transform::from_xyz(origin.x, origin.y, 0.1),
                ElevationChunk,
            ))
            .id();
        loaded.0.insert(*coord, entity);
    }

    let stale: Vec<IVec2> = loaded
        .0
        .keys()
        .filter(|k| !desired.contains(k))
        .copied()
        .collect();
    for coord in stale {
        if let Some(entity) = loaded.0.remove(&coord) {
            commands.entity(entity).despawn();
        }
    }
}
