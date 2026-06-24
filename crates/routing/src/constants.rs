use bevy::math::IVec2;

/// Default grid spacing for the search, in world units. Mirrors `world`'s `ELEVATION_CELL` so a
/// default-`step` search lines up with the height cells (no resampling within a cell). Copied, not
/// imported: keeping this leaf free of a `world` dependency is worth one duplicated literal — if
/// `ELEVATION_CELL` ever changes, revisit this.
pub const DEFAULT_STEP: f32 = 10.0;

/// Fixed-point scale turning float edge costs into the integer units `astar` needs (`Ord + Zero`).
/// 1000 ⇒ millis of a world unit — finer than any geometry we route over, coarse enough that a
/// continent-spanning path's summed `u64` cost can't overflow.
pub const COST_SCALE: f32 = 1000.0;

/// Ramer–Douglas–Peucker tolerance, as a multiple of `step`. 8-connected A* emits diagonals as a
/// single-cell staircase whose deviation from its chord is `<= step/2`; a tolerance near one step
/// flattens that staircase into a clean diagonal while preserving genuine bends.
pub const SIMPLIFY_TOLERANCE_STEPS: f32 = 1.0;

/// Minimum start↔goal separation, in world units, that `find_path` will route. Below this the
/// endpoints are effectively the same point: there is nothing to connect, and a zero-length result
/// would trip the downstream spline builder's positive-length assertion. Matches rail's
/// `MIN_SEGMENT_LEN`; kept here so the contract is enforced at the routing seam.
pub const MIN_ENDPOINT_SEP: f32 = 1e-3;

/// The eight grid neighbours (orthogonal + diagonal). Order is irrelevant to correctness.
pub const NEIGHBOURS: [IVec2; 8] = [
    IVec2::new(1, 0),
    IVec2::new(-1, 0),
    IVec2::new(0, 1),
    IVec2::new(0, -1),
    IVec2::new(1, 1),
    IVec2::new(1, -1),
    IVec2::new(-1, 1),
    IVec2::new(-1, -1),
];
