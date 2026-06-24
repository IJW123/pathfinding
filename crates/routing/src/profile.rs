use crate::constants::DEFAULT_STEP;

/// Tuning for one class of route. Rail passes a strict `max_grade`, roads a looser one, off-road
/// agents looser still — the algorithm is otherwise identical.
#[derive(Debug, Clone, Copy)]
pub struct PathProfile {
    /// Hard cutoff on `|Δheight| / horizontal_distance` per step. Edges steeper than this are
    /// impassable; the route goes around or [`find_path`](crate::find_path) returns `None`. Must be
    /// `> 0` — a non-positive cutoff would forbid every non-flat edge and the cost divides by it.
    pub max_grade: f32,
    /// How strongly to prefer flatter ground among *legal* edges. `0.0` ⇒ pure shortest path;
    /// larger ⇒ the route detours toward gentler slopes. Scales the per-edge cost by
    /// `1 + grade_cost_weight * grade / max_grade`.
    pub grade_cost_weight: f32,
    /// Grid spacing for the search, in world units. Smaller ⇒ finer routes, more nodes. Defaults to
    /// [`DEFAULT_STEP`] (the height-cell size), which aligns the search grid with the terrain grid.
    pub step: f32,
}

impl Default for PathProfile {
    fn default() -> Self {
        Self {
            max_grade: 0.05,
            grade_cost_weight: 1.0,
            step: DEFAULT_STEP,
        }
    }
}
