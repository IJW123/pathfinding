use bevy::asset::RenderAssetUsages;
use bevy::color::{ColorToComponents, LinearRgba};
use bevy::mesh::{Mesh, PrimitiveTopology};

use world::elevation::contour::data::ContourLine;

use crate::elevation::contour::style::ContourStyle;

#[must_use]
pub fn contour_lines_to_mesh(lines: &[ContourLine], style: &ContourStyle) -> Mesh {
    let total = lines.len();
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut colors: Vec<[f32; 4]> = Vec::new();

    for (idx, line) in lines.iter().enumerate() {
        let t = if total <= 1 {
            0.5
        } else {
            idx as f32 / (total - 1) as f32
        };
        let rgba = LinearRgba::from(style.color_for_t(t)).to_f32_array();

        for seg in &line.segments {
            positions.push([seg.a.x, seg.a.y, 0.0]);
            positions.push([seg.b.x, seg.b.y, 0.0]);
            colors.push(rgba);
            colors.push(rgba);
        }
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::LineList,
        RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh
}
