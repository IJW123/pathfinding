use bevy::prelude::*;

use world::elevation::chunk_events::ChunkLoaded;
use world::elevation::contour::extract::extract_contours;
use world::elevation::height_fn::HeightFn;

use crate::elevation::contour::cache::ContourCache;
use crate::elevation::contour::levels::ContourLevels;
use crate::elevation::contour::mesh::contour_lines_to_mesh;
use crate::elevation::contour::style::ContourStyle;

#[expect(clippy::too_many_arguments, reason = "Bevy system; each param is a distinct resource")]
pub fn render_contours_on_chunk_loaded(
    mut commands: Commands,
    mut events: MessageReader<ChunkLoaded>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut cache: ResMut<ContourCache>,
    height: Res<HeightFn>,
    levels: Res<ContourLevels>,
    style: Res<ContourStyle>,
) {
    let material = cache
        .material
        .get_or_insert_with(|| materials.add(ColorMaterial::from(Color::WHITE)))
        .clone();

    for ev in events.read() {
        let mesh = cache
            .meshes
            .entry(ev.coord)
            .or_insert_with(|| {
                let lines = extract_contours(ev.coord, &height, &levels.0);
                meshes.add(contour_lines_to_mesh(&lines, &style))
            })
            .clone();
        commands
            .entity(ev.entity)
            .insert((Mesh2d(mesh), MeshMaterial2d(material.clone())));
    }
}
