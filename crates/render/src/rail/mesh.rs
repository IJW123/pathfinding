use bevy::asset::RenderAssetUsages;
use bevy::math::Vec2;
use bevy::mesh::{Mesh, PrimitiveTopology};

/// Build a `LineStrip` mesh tracing the smoothed track centerline. Same cheap GPU-line technique as
/// `contour_lines_to_mesh`, applied to an authored path rather than an extracted iso-contour — so it
/// reuses the approach, not marching squares. A 1px line: thickness/zoom-scaling is a later concern.
#[must_use]
pub fn track_line_mesh(points: &[Vec2]) -> Mesh {
    let positions: Vec<[f32; 3]> = points.iter().map(|p| [p.x, p.y, 0.0]).collect();
    let mut mesh = Mesh::new(
        PrimitiveTopology::LineStrip,
        RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh
}
