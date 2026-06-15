# Move the terrain recipe (incl. map size) to the `level` crate

> Supersedes the map-size-only draft. Same task, expanded scope after audit: `AUTHORED_FEATURES`
> and the feature-population params were also level setup stuck in `world`.

## Context

Map size (`MAP_HALF_EXTENT`) and the terrain *content* — the hand-placed `AUTHORED_FEATURES`
(`placement.rs:14`) and the procedural population params (`FEATURE_SEED`, `HILL_COUNT`/ranges,
`MOUNTAIN_COUNT`/ranges in `elevation/constants.rs`) — currently live in `world`. That's level
layout baked into the terrain engine. We want one knob, owned by `level`: a level defines *what's on
its map*; `world` is a pure engine that turns a recipe into a `HeightField`.

A literal move is impossible (`level → world`; importing back = cycle), so invert via a runtime
resource: **`world` defines the `TerrainConfig` type, `level` authors the value and inserts it.**
Fully injected — `world` holds no recipe values; missing config = loud panic.

Ownership split:
- **world keeps the *how* (engine):** `fbm` noise, `flat_base`, `feature_value`, `FeatureSpec`,
  `HeightField` sampling/gradient, contour extraction, chunk math, and shape/quality tuning —
  `ELEVATION_CELL`, `ELEV_CHUNK_CELLS`, `HEIGHT_MIN/MAX`, all `FBM_*`, `FLAT_AMP/FREQ`,
  `HILL_ROUGHNESS`, `HILL_DETAIL_FREQ`.
- **level authors the *what* (recipe):** map extent, placement seed, hill/mountain populations, and
  the authored feature list.

Terrain isn't dynamically chunked (one monolithic immutable `HeightField`; `map_chunk_coords` is a
fixed tiling for contour rendering), so the recipe feeds construction-time logic only.

## Changes

### `world` — define the recipe type (the level seam)

- **New `crates/world/src/elevation/config.rs`**:
  ```rust
  #[derive(Resource, Clone)]
  pub struct TerrainConfig {
      pub half_extent: f32,
      pub seed: u32,
      pub hills: FeaturePopulation,
      pub mountains: FeaturePopulation,
      pub authored: Vec<FeatureSpec>,   // FeatureSpec from elevation::generation::feature
  }

  #[derive(Clone, Copy)]
  pub struct FeaturePopulation {
      pub count: u32,
      pub radius_min: f32,
      pub radius_max: f32,
      pub height_min: f32,
      pub height_max: f32,
  }
  ```
  Named fields (no `(f32,f32)` tuples) — also tidies the existing positional-tuple ranges in
  `random_features`, per the project's no-tuple-indexing rule.
- **`elevation/mod.rs`**: add `pub mod config;`.

### `world` — engine consumes the recipe instead of consts

- **`elevation/generation/placement.rs`**: delete `AUTHORED_FEATURES` and the population-const
  imports. `all_features(config: &TerrainConfig) -> Vec<FeatureSpec>` = `config.authored` +
  `random_features(config.seed, &config.hills, config.half_extent)` +
  `random_features(config.seed ^ MOUNTAIN_STREAM_MASK, &config.mountains, config.half_extent)`.
  `random_features(seed, pop: &FeaturePopulation, half_extent: f32)`. Keep the mountain-stream mask
  (`0xA5A5_A5A5`) as a local const — it's engine detail (how a second stream is derived), not recipe.
- **`elevation/height_field.rs`**: replace the `Default` impl with
  `pub fn new(config: &TerrainConfig) -> Self` — same body, `config.half_extent` for dims/origin and
  `all_features(config)` for stamping. The `#[cfg(test)] from_parts` seam is unchanged.
- **`elevation/chunk_coord.rs`**: `map_chunk_coords(half_extent: f32)`; drop the const import.
- **`elevation/constants.rs`**: remove `FEATURE_SEED`, `HILL_COUNT`, `HILL_RADIUS_*`,
  `HILL_HEIGHT_*`, `MOUNTAIN_*` (now recipe). Keep everything else (resolution, clamps, noise,
  detail).
- **`elevation/plugin.rs`**: replace `init_resource::<HeightField>()` with a **`PreStartup`** system:
  ```rust
  fn build_height_field(mut commands: Commands, config: Res<TerrainConfig>) {
      commands.insert_resource(HeightField::new(&config));
  }
  ```
  `PreStartup` guarantees `HeightField` exists before every `Startup` consumer
  (`spawn_contour_tiles`). `TerrainConfig` is inserted at level's plugin-build (below), so it's
  present before `PreStartup` regardless of plugin add-order. Missing config → panic on
  `Res<TerrainConfig>` (intended fail-fast).
- **`lib.rs`**: remove `pub mod constants;`. **Delete `crates/world/src/constants.rs`**
  (`MAP_HALF_EXTENT` was its last entry; `WALL_*` already left in p17).

### `render` — pass extent to the contour tiler

- **`elevation/contour/render.rs`**: `spawn_contour_tiles` gains `config: Res<TerrainConfig>` and
  calls `map_chunk_coords(config.half_extent)`. Import `world::elevation::config::TerrainConfig`.

### `level` — author the recipe, insert the resource

- **New `crates/level/src/constants.rs`**: the authored values — `MAP_HALF_EXTENT: f32 = 2000.0`,
  the population scalars (counts/ranges), `FEATURE_SEED`, and `AUTHORED_FEATURES: &[FeatureSpec]`
  (the two hand-placed features, moved verbatim). Add `pub mod constants;` to `lib.rs`.
- **New `crates/level/src/terrain.rs`**: `fn level_terrain() -> TerrainConfig` assembling the consts
  into the recipe (keeps the literal out of the plugin). Add `pub mod terrain;` to `lib.rs`.
- **`level/src/plugin.rs`**: `LevelPlugin::build` adds
  `app.insert_resource(level_terrain());` alongside `spawn_level`.
- **`level/src/spawn.rs`**: import `MAP_HALF_EXTENT` from `crate::constants` (not `world`);
  `boundary_walls(MAP_HALF_EXTENT, WALL_THICKNESS)` unchanged. Walls and the recipe both read the one
  `level` const → single source.

## Test edits (flagging per the no-silent-test-edits rule — all mechanical, none deleted)

- `placement.rs` tests: rebuild around `FeaturePopulation` literals + a small `TerrainConfig`; thread
  `2000.0`. `all_features_counts_authored_plus_random` asserts against the test config's authored len
  + counts (same shape, values from the local config instead of moved consts).
- `chunk_coord.rs` tests: `map_chunk_coords()` → `map_chunk_coords(2000.0)`; 14×14 assertion stands.
- `height_field.rs`, `feature.rs` tests: unaffected (use `from_parts` / kept consts).

## Files touched

- `crates/world/src/lib.rs` + delete `constants.rs`; new `elevation/config.rs`
- `crates/world/src/elevation/{mod.rs, chunk_coord.rs, height_field.rs, plugin.rs, generation/placement.rs}`
- `crates/render/src/elevation/contour/render.rs`
- `crates/level/src/{lib.rs, plugin.rs, spawn.rs}`; new `constants.rs`, `terrain.rs`
- (housekeeping) rename plan file from the legacy `p18_map_size_to_level.md`

## Verification

1. `cd /home/isaak/RustroverProjects/pathfinding && ./bin/housekeeping.sh` — clean clippy + fmt.
2. `cargo test -p world -p level -p render` — threaded tests pass; chunk count still 14×14.
3. `cargo run -p pathfinding` — terrain (flat base + hills + the two authored features), contours, and
   the four walls render **identically** to today; player slope/elevation behavior unchanged.
4. Sanity: bump a value in `level`'s recipe (e.g. `HILL_COUNT`, or move an authored feature) and
   confirm the terrain changes — proving the recipe now lives in `level`.

## Explicitly not doing

- Not moving engine tuning (noise, falloff, resolution, clamps) — that's *how* terrain looks, world's
  job.
- Not adding runtime/multi-map switching — just relocating ownership of the single recipe.
- No world-side default recipe (fully injected, per decision).
