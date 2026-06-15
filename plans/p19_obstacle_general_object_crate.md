# p19 — Obstacle as general object crate + form/size separation

## Goal
Make `obstacle` the general home for non-player objects, and fix the separation-of-concerns
leak: the crate currently owns **form *and* size** (size frozen into hand-authored vertex
arrays). After this, the crate owns **form only**; `level` owns **size + center**.

Decisions:
- Rename crate `obstacles` → `obstacle` (singular), general non-player-object home.
- `Obstacle` stays welded to `Solid` (`#[require(Solid)]` unchanged).
- D1: keep authored irregular silhouettes, unit-normalized.
- D2: shape constructors return `Collider` (internal `expect`).
- D3: sizes become `level` constants.

## Part A — Rename `obstacles` → `obstacle` (plain mv, no git mv)
- `mv crates/obstacles crates/obstacle`
- `crates/obstacle/Cargo.toml`: `name = "obstacle"`
- Workspace `Cargo.toml`: member `"crates/obstacle"`; dep `obstacle = { path = "crates/obstacle" }`
- `crates/level/Cargo.toml`, `crates/render/Cargo.toml`: dep key `obstacles` → `obstacle`
- Code refs `use obstacles::` → `use obstacle::`: level/spawn.rs, level/tests/spawn_level.rs,
  render/src/obstacle/plugin.rs (render's local `crate::obstacle` module coexists, no conflict).

## Part B — Form-only shapes (`crates/obstacle/src/shape.rs`)
- Private unit silhouettes (circumradius 1.0, local origin-centered): TRIANGLE_UNIT, QUAD_UNIT,
  PENTAGON_UNIT — current shapes scaled down.
- Pub constructors returning `Collider`: `triangle(size)`, `quad(size)`, `pentagon(size)`
  (scale unit pts by size → `Collider::convex(...).expect(...)`), `circle(radius)` re-expose.
- `constants.rs`: keep OBSTACLE_Z, WALL_Z, WALL_THICKNESS. Remove CIRCLE_RADIUS + *_POINTS.
- `lib.rs`: add `pub mod shape;`.

## Part C — `level` sets size + center
- `spawn.rs`: four interior spawns use `obstacle::shape::{...}(size)` + Transform center.
- `level/constants.rs`: add CIRCLE_RADIUS, TRIANGLE_SIZE, QUAD_SIZE, PENTAGON_SIZE.

## Tests
- spawn_level.rs: import-path change only.
- shape.rs: hull vertex counts + linear size scaling of render_size.

## Housekeeping
`./bin/housekeeping.sh`, fix all warnings.
