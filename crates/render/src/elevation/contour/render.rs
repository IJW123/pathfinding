use bevy::prelude::*;

use world::elevation::chunk_events::ChunkLoaded;
use world::elevation::contour::extract::extract_contours;
use world::elevation::height_fn::HeightFn;

use crate::elevation::components::ContourGeometry;
use crate::elevation::contour::levels::ContourLevels;
use crate::elevation::contour::mesh::contour_lines_to_mesh;
use crate::elevation::contour::style::ContourStyle;

pub fn render_contours_on_chunk_loaded(
    mut commands: Commands,
    mut events: MessageReader<ChunkLoaded>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    height: Res<HeightFn>,
    levels: Res<ContourLevels>,
    style: Res<ContourStyle>,
) {
    for ev in events.read() {
        let lines = extract_contours(ev.coord, &height, &levels.0);
        let mesh = contour_lines_to_mesh(&lines, &style);
        let mesh_handle = meshes.add(mesh);
        let mat_handle = materials.add(ColorMaterial::from(Color::WHITE));
        commands.entity(ev.entity).insert((
            Mesh2d(mesh_handle),
            MeshMaterial2d(mat_handle),
            ContourGeometry { lines },
        ));
    }
}
