use bevy::asset::RenderAssetUsages;
use bevy::math::Vec2;
use bevy::mesh::{Indices, Mesh, PrimitiveTopology};

/// Filled mesh for a convex hull via fan triangulation (`v0, vi, vi+1`). Points come from
/// `ConvexHull::points()`, so >= 3 vertices and CCW winding (hence consistent triangle winding)
/// are guaranteed by the type. POSITION-only plus indices — all an untextured `ColorMaterial`
/// fill needs (the contour renderer ships an even sparser attribute set).
#[must_use]
pub fn convex_mesh(points: &[Vec2]) -> Mesh {
    let positions: Vec<[f32; 3]> = points.iter().map(|p| [p.x, p.y, 0.0]).collect();
    let last = points.len() as u32 - 1;
    let indices: Vec<u32> = (1..last).flat_map(|i| [0, i, i + 1]).collect();

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fan_triangulation_counts() {
        let pentagon = [
            Vec2::new(0.0, 1.0),
            Vec2::new(-1.0, 0.3),
            Vec2::new(-0.6, -0.8),
            Vec2::new(0.6, -0.8),
            Vec2::new(1.0, 0.3),
        ];
        let mesh = convex_mesh(&pentagon);
        assert_eq!(mesh.count_vertices(), 5);
        let indices = mesh.indices().expect("indices present");
        assert_eq!(indices.len(), 3 * (5 - 2));
    }
}
