use bevy::prelude::*;

use world::elevation::chunk_coord::{chunk_origin_world, map_chunk_coords};
use world::elevation::config::TerrainConfig;
use world::elevation::contour::extract::extract_contours;
use world::elevation::height_field::HeightField;

use crate::elevation::components::ContourTile;
use crate::elevation::contour::levels::ContourLevels;
use crate::elevation::contour::mesh::contour_lines_to_mesh;
use crate::elevation::contour::style::ContourStyle;

/// Build every map tile's contour mesh once at startup. The map is fixed and the
/// `HeightField` immutable, so geometry never changes; tiles with no contours (flat ground)
/// are skipped. Bevy frustum-culls offscreen tiles — no streaming needed.
pub fn spawn_contour_tiles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    height: Res<HeightField>,
    levels: Res<ContourLevels>,
    style: Res<ContourStyle>,
    config: Res<TerrainConfig>,
) {
    let material = materials.add(ColorMaterial::from(Color::WHITE));
    for coord in map_chunk_coords(config.half_extent) {
        let lines = extract_contours(coord, &height, &levels.0);
        if lines.iter().all(|line| line.segments.is_empty()) {
            continue;
        }
        let origin = chunk_origin_world(coord);
        commands.spawn((
            Transform::from_xyz(origin.x, origin.y, 0.1),
            Mesh2d(meshes.add(contour_lines_to_mesh(&lines, &style))),
            MeshMaterial2d(material.clone()),
            ContourTile,
        ));
    }
}
