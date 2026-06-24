use bevy::math::{Rect, Vec2};

/// The terrain seam this crate routes over. A producer of routes (rail, road, agent) holds an
/// implementor — typically a heightmap resource — and hands it to [`find_path`](crate::find_path).
///
/// Keeping it a trait, rather than a concrete heightmap type, is what makes `routing` a terrain-
/// agnostic leaf: the world crate implements this for its `HeightField`, tests implement it for a
/// ramp or a cliff, and neither leaks into the algorithm.
pub trait ElevationSampler {
    /// World-space elevation at `p`. Implementors clamp out-of-bounds queries to the edge.
    fn height(&self, p: Vec2) -> f32;

    /// World-space extent of the routable area. The search is confined to nodes inside this rect,
    /// so the route never wanders onto terrain the field can't meaningfully sample.
    fn bounds(&self) -> Rect;
}
