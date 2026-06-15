# Fold walls into the obstacles crate

## Context

Boundary-wall code is scattered across the `world` crate (`Wall` marker in `components.rs`,
`wall()`/`boundary_walls()` in `bundle.rs`, `WALL_THICKNESS`/`WALL_Z` in `constants.rs`) plus a
dedicated `render/wall/` plugin. A wall is functionally just a static OBB obstacle — same
`Solid`+`Static`, and the obstacle renderer already draws OBB shapes. Keeping walls as a separate
concept in `world` (whose real job is terrain/elevation) is messy. The obstacles crate is the
natural home.

This follows on from p16, which centralized *placement* into the `level` crate. This change
centralizes the *wall kind* itself into `obstacles`.

**Chosen approach (Wall = tagged Obstacle):** walls become real `Obstacle`s. `boundary_walls()`
returns `static_obstacle` bundles plus a `Wall` tag. One unified obstacle renderer special-cases the
`Wall` tag for the gray color; the separate wall render plugin is deleted. Walls stay queryable via
`Wall` and now count as obstacles. No visual regression (same size/color; the wall's Sprite becomes a
Mesh2d, which is geometrically identical for an OBB).

`MAP_HALF_EXTENT` stays in `world` — it's map/terrain config shared with elevation streaming
(`chunk_coord.rs`, `height_field.rs`, `generation/placement.rs`), and `boundary_walls` already takes
the extent as a parameter, so `obstacles` stays free of a `world` dependency.

## Changes

### `obstacles` crate (destination)

- **`components.rs`**: add `Wall` marker. Plain tag — `Solid`/`Static`/`Obstacle` come from the
  `static_obstacle` bundle in `wall()`, so no `#[require(...)]`. Doc it as the boundary-wall sub-kind.
- **`constants.rs`**: add `WALL_THICKNESS` (20.0) and `WALL_Z` (0.0), moved from `world/constants.rs`.
- **`bundle.rs`**: add `wall(center, half_extents)` and `boundary_walls(half_extent, thickness)`,
  moved from `world/bundle.rs`. Rewrite `wall()` to compose the existing `static_obstacle`:
  `(static_obstacle(Transform::from_xyz(center.x, center.y, WALL_Z), Collider::obb(half_extents)), Wall)`.
  `boundary_walls` body otherwise unchanged (4 walls derived from extent + thickness). Relocate the
  two existing bundle tests here; adapt `wall_is_solid_and_static` to also assert `Obstacle` is present.

### `world` crate (source — strip wall code)

- **Delete `crates/world/src/bundle.rs`** (wall-only; tests move to obstacles).
- **Delete `crates/world/src/components.rs`** (contained only `Wall`).
- **`lib.rs`**: drop `pub mod bundle;` and `pub mod components;`.
- **`constants.rs`**: remove `WALL_THICKNESS` and `WALL_Z`; keep `MAP_HALF_EXTENT`.

### `render` crate (unify renderers)

- **Delete `crates/render/src/wall/`** (constants.rs, mod.rs, plugin.rs).
- **`lib.rs`**: drop `pub mod wall;`.
- **`plugin.rs`**: remove the `WallRenderPlugin` import and its entry in `add_plugins`.
- **`obstacle/constants.rs`**: add `OBSTACLE_WALL_COLOR = Color::srgb(0.4, 0.4, 0.4)` (the old
  `WALL_SPRITE_COLOR`).
- **`obstacle/plugin.rs`**: add `Option<&Wall>` to the `Added<Obstacle>` query; pick
  `OBSTACLE_WALL_COLOR` when the `Wall` tag is present, else the existing static/dynamic logic. Walls
  have OBB colliders, so the existing `ColliderShape::Obb => Rectangle` arm renders them — no Sprite
  path needed.

### `level` crate (call site)

- **`spawn.rs`**: import `boundary_walls`/`WALL_THICKNESS` from `obstacles` instead of `world`
  (keep `MAP_HALF_EXTENT` from `world`). The `boundary_walls(MAP_HALF_EXTENT, WALL_THICKNESS)` call is
  otherwise unchanged.
- **`tests/spawn_level.rs`**: change `use world::components::Wall;` → `use obstacles::components::Wall;`.
  Walls are now also `Obstacle`s, so the `query::<&Obstacle>().count()` assertion becomes **8**
  (4 walls + 4 obstacles), not 4. The `&Wall` count stays 4 and pushable count stays 2.

> ⚠️ Test edits: this touches two test files. `spawn_level.rs` needs the import swap **and** the
> obstacle-count assertion bumped 4 → 8 (a real semantic consequence of walls becoming obstacles).
> The relocated `bundle.rs` tests are moved, not deleted. Flagging per the no-silent-test-edits rule —
> confirm you're good with the count change before implementation.

## Files touched

- `crates/obstacles/src/{components.rs, constants.rs, bundle.rs}`
- `crates/world/src/{lib.rs, constants.rs}` + delete `bundle.rs`, `components.rs`
- `crates/render/src/{lib.rs, plugin.rs}` + `obstacle/{constants.rs, plugin.rs}` + delete `wall/`
- `crates/level/src/spawn.rs`, `crates/level/tests/spawn_level.rs`

## Verification

1. `cd /home/isaak/RustroverProjects/pathfinding && ./bin/housekeeping.sh` — clean clippy + fmt, no warnings.
2. `cargo test -p obstacles -p level` — relocated bundle tests + adapted spawn_level tests pass
   (4 walls, 8 obstacles, 2 pushable).
3. `cargo run -p pathfinding` — four gray boundary walls render at the map edge exactly as before;
   interior obstacles unchanged; player still collides with walls.

## Explicitly not doing

- Not eliminating the `Wall` concept (an alternative considered): walls keep a queryable tag and their
  distinct gray color.
- Not moving `MAP_HALF_EXTENT` out of `world` — it's terrain config, and moving it would make
  `obstacles` depend on `world`.
