use bevy::math::{IVec2, Rect, Vec2};

/// Square lattice over the routable rect: integer node `(0,0)` sits at `bounds.min`, neighbours are
/// `step` world units apart on each axis. Pure geometry — no elevation here.
#[derive(Clone, Copy)]
pub struct Grid {
    origin: Vec2,
    step: f32,
    bounds: Rect,
}

impl Grid {
    #[must_use]
    pub fn new(bounds: Rect, step: f32) -> Self {
        Self {
            origin: bounds.min,
            step,
            bounds,
        }
    }

    /// World position of a node's center.
    #[must_use]
    pub fn to_world(self, node: IVec2) -> Vec2 {
        self.origin + node.as_vec2() * self.step
    }

    /// Nearest node to a world position. Out-of-bounds positions snap to the nearest *node*, which
    /// `clamp` then pins into the grid — so a far-off endpoint resolves to the closest reachable
    /// node rather than a search that never terminates.
    #[must_use]
    pub fn to_node(self, p: Vec2) -> IVec2 {
        let local = (p - self.origin) / self.step;
        self.clamp(local.round().as_ivec2())
    }

    /// Whether a node's center lies within the routable rect.
    #[must_use]
    pub fn in_bounds(&self, node: IVec2) -> bool {
        self.bounds.contains(self.to_world(node))
    }

    /// Pin a node so its center stays inside the routable rect.
    #[must_use]
    pub fn clamp(&self, node: IVec2) -> IVec2 {
        let max = ((self.bounds.max - self.origin) / self.step)
            .floor()
            .as_ivec2();
        node.clamp(IVec2::ZERO, max)
    }
}
