# Plan: p10 — obstacle objects (circle + convex polygons) in a new crate

## Context

The p9 SAT rewrite gave collision real shape support (OBB / convex / circle + manifolds), but
nothing in the world actually spawns a non-box shape yet — gameplay still only uses the square
player and square walls. We want to populate the map with a circle and a few convex polygons to
(a) exercise the new narrow phase against real rotated/round/convex geometry, and (b) start a
general "placed objects" system distinct from terrain and boundaries.

Decisions (from the user):
- **New `obstacles` crate** for the spawn logic — placed objects are their own concern, separate
  from terrain/boundary geometry in `world`.
- **Proper mesh rendering** (not placeholder sprites) so shapes are visually accurate — reuse the
  existing `Mesh2d` + `ColorMaterial` pattern from the contour renderer.
- **Both static and dynamic** obstacles — mix immovable (`Solid + Static`) and pushable (`Solid`)
  so we cover the full response table including the dynamic-dynamic 0.5/0.5 split.

This mirrors the existing wall split exactly: spawn logic lives crate-side
(`world/src/wall.rs` → now `obstacles/src/spawn.rs`), rendering lives render-side
(`render/src/wall/` → now `render/src/obstacle/`). Rendering reads from the obstacle/collision
logic, never the reverse.

## Architecture

```
obstacles (new crate)            render (existing)              app
  components.rs  Obstacle    ◄── obstacle/plugin.rs ──┐
  constants.rs   geometry         obstacle/mesh.rs    │   main.rs adds
  spawn.rs       setup system     obstacle/constants  │   ObstaclesPlugin
  plugin.rs      ObstaclesPlugin  (colors)            │
        ▲                                             ▼
        └──────────── depends on ──► collision (Collider, Solid, Static, ColliderShape)
```

Dependency direction: `render → obstacles → collision`. No cycle. Obstacles never depends on
render. **Single source of truth for geometry: the `Collider` component.** The render system reads
`collider.shape` to build the mesh, so collider and mesh can never drift — there is no second copy
of the points.

## New crate: `obstacles`

- `crates/obstacles/Cargo.toml` — deps `bevy = { workspace = true }`, `collision = { workspace = true }`.
- `crates/obstacles/src/lib.rs` — `pub mod components; pub mod constants; pub mod plugin; pub mod spawn;`
- `components.rs` — `#[derive(Component)] pub struct Obstacle;` (marker; render keys off it via `Added<Obstacle>`).
- `constants.rs` — all geometry literals (per the per-module constants policy; `Vec2::new` is `const`):
  - circle: center + radius, e.g. `CIRCLE_CENTER: Vec2`, `CIRCLE_RADIUS: f32`.
  - convex hulls as `const [Vec2; N]`, **local-space, CCW, convex**: a triangle, an irregular convex
    quad, a pentagon. Keep them origin-centered. Winding/convexity are enforced by `ConvexHull::new`
    (p9 validation): CW input is auto-reversed, concave/degenerate input errors at startup — author
    CCW for readability, not safety.
  - per-obstacle placement: world position + z (`OBSTACLE_Z`, between contour `0.1` and player `1.0`,
    e.g. `0.5`) + tilt angle for at least one shape (to exercise rotated SAT).
- `spawn.rs` — `setup_obstacles(mut commands)` (Startup). For each obstacle, spawn the wall-style
  bundle: `Transform` (pos, z, optional `Quat::from_rotation_z(tilt)`), `Obstacle`,
  `Collider::circle(..)` / `Collider::convex(points.to_vec()).expect("valid obstacle hull")` —
  `convex` returns `Result<_, HullError>` since p9 validation; the expect is the authoring safety
  net (clean startup panic with a descriptive error) — `Solid`, and `Static` for the immovable
  ones. Concretely:
  - circle near middle — **static**.
  - tilted triangle — **static** (rotated, proves rotated poly/poly + poly/circle).
  - convex quad — **dynamic** (pushable).
  - pentagon — **dynamic** (pushable; covers dynamic-dynamic if shoved into the quad).
  Place them clear of the player spawn at the origin.
- `plugin.rs` — `ObstaclesPlugin` → `app.add_systems(Startup, setup_obstacles)`.

## Render side: `render/src/obstacle/`

- `mod.rs` — `pub mod constants; pub mod mesh; pub mod plugin;` (no code in mod.rs).
- `constants.rs` — `OBSTACLE_STATIC_COLOR`, `OBSTACLE_DYNAMIC_COLOR` (`Color`), so you can see at a
  glance which obstacles are pushable.
- `mesh.rs` — `pub fn convex_mesh(points: &[Vec2]) -> Mesh`: fan-triangulate a convex hull
  (`(v0, vi, vi+1)` for `i in 1..n-1`), `PrimitiveTopology::TriangleList`. Input comes from
  `ConvexHull::points()`, so CCW winding is guaranteed by the type and the fan triangulation is
  deterministic (all triangles CCW). Insert `ATTRIBUTE_POSITION`
  + indices (the minimum a filled, untextured `ColorMaterial` needs). The contour renderer proves
  `ColorMaterial` is happy with a sparse attribute set — it ships only POSITION + COLOR on a
  `LineList`, no normals/UVs/indices (`contour/mesh.rs`). So do **not** assume NORMAL/UV_0 are
  required; add `ATTRIBUTE_NORMAL` (+Z) / `ATTRIBUTE_UV_0` only if the pipeline actually rejects the
  POSITION-only mesh (it shouldn't for untextured fill). Pure function, unit-testable.
- `plugin.rs` — `ObstacleRenderPlugin` → `app.add_systems(Update, attach_obstacle_mesh)` (**Update,
  not Startup** — mirrors `WallRenderPlugin`; spawn happens on `Startup`, so `Added<Obstacle>` only
  fires correctly if the attach runs on a later schedule). System
  `attach_obstacle_mesh(commands, meshes: ResMut<Assets<Mesh>>, materials: ResMut<Assets<ColorMaterial>>,
   query: Query<(Entity, &Collider, Option<&Static>), Added<Obstacle>>)`:
  - color = `Static` present ? static color : dynamic color → `materials.add(ColorMaterial::from(color))`.
    One-shot decision at `Added<Obstacle>` time — won't track a later `Static` toggle; fine for p10.
  - mesh by `collider.shape`:
    - `ColliderShape::Circle { radius }` → `meshes.add(Circle::new(radius))` (Bevy primitive).
    - `ColliderShape::Convex { hull }` → `meshes.add(convex_mesh(hull.points()))`.
    - `ColliderShape::Obb { half_extents }` → `meshes.add(Rectangle::new(2*hx, 2*hy))` (completeness;
      no Obb obstacles spawned today, but the dispatch is total).
  - insert `Mesh2d(..)` + `MeshMaterial2d(..)`. The entity's `Transform` (set in spawn) handles world
    position + rotation; meshes are built in local space, so off-center/rotated hulls render correctly
    (this sidesteps the sprite-centering caveat from p9 — meshes use the real vertices).
- Register `ObstacleRenderPlugin` in `render/src/plugin.rs` alongside the existing three.

## Wiring

- `Cargo.toml` (workspace) — add `"crates/obstacles"` to `members`; add
  `obstacles = { path = "crates/obstacles" }` to `[workspace.dependencies]`.
- `crates/render/Cargo.toml` — add `obstacles = { workspace = true }`.
- `crates/app/src/main.rs` — `use obstacles::plugin::ObstaclesPlugin;` and add it to the `add_plugins`
  tuple. App Cargo.toml gains `obstacles = { workspace = true }`.

## Module rules compliance

No code in any `mod.rs` (markers/fns live in `components.rs`/`spawn.rs`/`mesh.rs`). Geometry literals
in `obstacles/constants.rs`; render colors in `render/src/obstacle/constants.rs`. Reuses
`Collider::circle`/`convex`, `Solid`, `Static`, `ColliderShape` from p9 and the contour renderer's
`Mesh2d` + `ColorMaterial` pattern — no new collision or rendering machinery invented.

## Implementation order

1. `obstacles` crate skeleton: `Cargo.toml`, `lib.rs`, `components.rs`, `constants.rs` (geometry),
   `spawn.rs`, `plugin.rs`. Add to workspace members + deps.
2. `cargo build -p obstacles` green.
3. `render/src/obstacle/` (`mesh.rs` + `convex_mesh` test, `constants.rs`, `plugin.rs`); register in
   `render/plugin.rs`; add `obstacles` dep to render Cargo.toml.
4. Wire `ObstaclesPlugin` into `app/main.rs` (+ app Cargo.toml dep).
5. `./bin/housekeeping.sh`, fix warnings.

## Verification

- `cargo test --workspace` — p9 collision tests stay green; new `convex_mesh` test (vertex count =
  N, index count = 3*(N-2)) passes; everything compiles.
- `./bin/housekeeping.sh` — clippy + fmt clean.
- `cargo run -p pathfinding` — near the middle: a filled circle and several convex polygons render
  with **accurate geometry** (circle is round, polygons are their real shape, the tilted one is
  visibly rotated), static vs dynamic distinguishable by color. Drive the player into them:
  - static circle/triangle push the player out cleanly (no tunneling/jitter) — rotated SAT working.
  - dynamic quad/pentagon get shoved; pushing one into another shows the dynamic-dynamic split.

## Risks / notes

- **Mesh attributes:** start with POSITION + indices. If (and only if) ColorMaterial/Mesh2d rejects
  it, add NORMAL (+Z) then UV_0 — POSITION/NORMAL/UV_0 + indices is the maximal known-good set. The
  contour renderer ships only POSITION + COLOR, so the bar is low; adjust attributes here, never the
  shape data.
- **Hull winding:** resolved by p9 hull validation (`plans/p9_convex_hull_validation.md`).
  `ConvexHull::new` runs in all builds: CW hulls are reversed losslessly, concave/degenerate hulls
  return `HullError`, and the spawn-site `.expect` turns authoring mistakes into a descriptive
  startup panic instead of silent wrong collisions.
- Spawn positions must avoid the origin (player spawn) so the player doesn't start embedded in an
  obstacle.