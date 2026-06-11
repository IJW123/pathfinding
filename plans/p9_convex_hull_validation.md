# P9: Convex Hull Validation

## Problem
- `Collider::convex` only `debug_assert`s convexity — concave hulls slide through in release and SAT gives garbage.
- CW winding is accepted while `edge_normals` docs claim CCW/outward.
- Fully-collinear (zero-area) point sets pass `is_convex` (sign never set).
- Collinearity epsilon compares raw cross products against absolute `COLLISION_EPSILON` — scale-dependent.

## Fix: validated `ConvexHull` newtype
New `crates/collision/src/hull.rs`:

- `ConvexHull { points: Vec<Vec2> }` — private field; invariant: >=3 points, convex, CCW, non-degenerate. `points()` accessor.
- `ConvexHull::new(points) -> Result<Self, HullError>` validates in all builds:
  1. `len() >= 3` → `TooFewPoints`
  2. Convexity: per-triple turn `sin = cross / (|ab|·|bc|)` — scale-free; mixed signs → `NotConvex`. Zero-length edges (duplicate points) contribute nothing (skip).
  3. No non-collinear triple at all → `Degenerate` (covers zero-area; no separate area threshold needed)
  4. CW (turn sign negative) → reverse points. Lossless auto-fix; makes "outward" edge normals unconditionally true.
- `HullError { TooFewPoints, NotConvex, Degenerate }` with `Display` + `Error` impls.

## Changes
- `shape.rs`: `Convex { hull: ConvexHull }` instead of raw `Vec<Vec2>`; type carries the proof, no re-checks downstream.
- `components.rs`: `Collider::convex(points) -> Result<Self, HullError>`; delete `is_convex` + debug_asserts; `render_size` uses `hull.points()`.
- `world.rs` `to_world`: `hull.points()`.
- `constants.rs`: `HULL_COLLINEAR_EPSILON` (sine-of-turn-angle threshold, dimensionless).
- `lib.rs`: add `hull` module.

## Decisions (agreed)
- `Result` constructor only, no panicking dual API; spawn code `.expect()`s.
- Reject concave input, don't auto-hullify (hides authoring bugs).
- Obb/Circle stay unvalidated — out of scope.

## Tests
- hull: too-few, concave rejected, collinear rejected, CW auto-reversed to CCW, valid CCW unchanged, duplicate consecutive points tolerated.
- Update `shape.rs` test constructing `Convex` directly.
