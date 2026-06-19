# p28 — Terrain recipe into the level asset (empty out level/constants.rs)

## Goal
Finish what p27 started. The only constants left in `level/src/constants.rs` are the **terrain
recipe** (`MAP_HALF_EXTENT`, `FEATURE_SEED`, `HILL_*`, `MOUNTAIN_*`, `AUTHORED_FEATURES`) — same
scattered-authored-data smell, different subsystem. Move them into `assets/level.ron` so the file
becomes the whole level (objects + map + terrain) and `level/src/constants.rs` is **deleted**.

## The ordering constraint that drives the design
`world::build_height_field` reads `Res<TerrainConfig>` in **`PreStartup`**, and `ElevationPlugin` is
added before `LevelPlugin` with **no cross-crate system ordering**. It works today only because
`LevelPlugin::build()` inserts `TerrainConfig` at **build() time** (before any schedule runs).

⇒ `TerrainConfig` must stay inserted at `build()`. So the unified spec must be **loaded at build()**,
not in a `PreStartup` system. **This revises p27**: the `PreStartup` `load_level_spec` system + its
`run_if(not(resource_exists))` seam are replaced by a build-time load guarded by a
`contains_resource::<LevelSpec>()` check. Net effect is *safer* — both `LevelSpec` and `TerrainConfig`
exist before any schedule, exactly as `TerrainConfig` does now. (A `PreStartup` loader for terrain
would race `build_height_field` — the elevation-lifecycle fragility we don't want to poke.)

## Schema (one file — `map_half_extent` is shared, so it can't split cleanly)
`level.ron` gains `map_half_extent` (top-level, single source of truth for walls **and** terrain) and
a `terrain` section:
```
(
  map_half_extent: 2000.0,
  terrain: (
    seed: 0x5EED1234,
    hills:     (count: 36, radius_min: 120.0, radius_max: 280.0, height_min: 18.0, height_max: 45.0),
    mountains: (count: 6,  radius_min: 350.0, radius_max: 600.0, height_min: 70.0, height_max: 100.0),
    authored: [ (center: (0.0, 0.0), radius: 220.0, height: 40.0),
                (center: (-900.0, 700.0), radius: 500.0, height: 95.0) ],
  ),
  obstacles: [ ... ],   // unchanged from p27
  storage: ( ... ),
  carrier: ( ... ),
)
```

## Layers (mirror p27 — raw serde → in-memory, one-way dep)
`world` has no serde (domain crate), so mirror its terrain types in the raw layer, same as
`RawCommodity` mirrors `Commodity`.

- `manifest.rs` (+): `RawTerrainSpec { seed, hills, mountains, authored }`,
  `RawFeaturePopulation { count, radius_min, radius_max, height_min, height_max }`,
  `RawFeatureSpec { center: (f32,f32), radius, height }`. `RawLevelSpec` gains `map_half_extent: f32`
  and `terrain: RawTerrainSpec`. (`FeaturePopulation`/`FeatureSpec` are owned by `world` serde-free,
  so they're mirrored, not shared — unlike `ObstacleShape` which `level` owns.)
  Add inherent mappers on the raw types, **exactly mirroring `RawCommodity::to_commodity`**:
  `RawFeaturePopulation::to_population(self) -> FeaturePopulation` and
  `RawFeatureSpec::to_feature(self) -> FeatureSpec` (tuple center → `Vec2`). The raw layer already
  imports a foreign domain type (`Commodity`) to return from `to_commodity`, so importing the `world`
  types here for the same purpose is consistent — it keeps the construction of foreign types out of
  `spec`'s `From` impls and the orphan boundary explicit (see below).
- `spec.rs` (+): in-memory `TerrainSpec { seed: u32, hills: FeaturePopulation, mountains:
  FeaturePopulation, authored: Vec<FeatureSpec> }` using the **real `world` types**
  (`FeaturePopulation`, `FeatureSpec`) — `level` already depends on `world`. `LevelSpec` gains
  `map_half_extent: f32` and `terrain: TerrainSpec`.
  **Orphan-rule trap — same one p27 caught for `RawCommodity`:** only `From<RawTerrainSpec> for
  TerrainSpec` is legal (local `Self`). Do **not** write `From<RawFeaturePopulation> for
  FeaturePopulation` or `From<RawFeatureSpec> for FeatureSpec` — both have a *foreign* `Self`
  (`world` types), so the orphan rule rejects them, exactly like `From<RawCommodity> for Commodity`.
  Inside `From<RawTerrainSpec>`, build the foreign fields via the `to_population`/`to_feature`
  inherent mappers (mirrors `stock.map(|(c,n)| (c.to_commodity(), n))`):
  `hills: raw.hills.to_population()`, `authored: raw.authored.into_iter().map(RawFeatureSpec::to_feature).collect()`.
- `LevelSpec::terrain_config(&self) -> TerrainConfig` — builds the `world` resource, setting
  `half_extent = self.map_half_extent` (keeps the single-source-of-truth the old `MAP_HALF_EXTENT`
  comment promised) and cloning `terrain.{seed,hills,mountains,authored}` across. Replaces
  `terrain.rs::level_terrain()`. `TerrainConfig`/`FeaturePopulation` already derive `Clone`, so this
  is a plain field copy — no new derives needed.

**Placement / module-doc honesty:** `LevelSpec` now spans objects **+ map + terrain**, but it still
lives in `objects/spec.rs` under a module whose doc (`objects.rs`) reads "per-object entity
composition." Don't restructure (minimal change), but **update the `objects.rs` module doc** so it
states `LevelSpec` is the whole-level root (objects + map size + terrain recipe), not just objects.
Leaving the doc claiming "per-object" while terrain lives there is the kind of quiet drift CLAUDE.md
warns against.

## Wiring
- `loader.rs`: `load_level_spec` changes from a Bevy *system* to a plain
  `pub fn load_level_spec() -> LevelSpec` (fs read + ron parse + `From`, fail-loud, same messages).
  Drop the now-unused `use bevy::prelude::*` / `Commands`, and fix the module + fn doc comments —
  they currently say "PreStartup load" / "The plugin guards it with `run_if(...)`", both untrue after
  this change. New wording: build-time load, guarded by the plugin's `contains_resource` check.
- `plugin.rs::build()`:
  ```
  if !app.world().contains_resource::<LevelSpec>() {
      app.insert_resource(load_level_spec());          // production: read file (or test pre-inserts)
  }
  let terrain = app.world().resource::<LevelSpec>().terrain_config();
  app.insert_resource(terrain).add_systems(Startup, spawn_level);
  ```
  No more `PreStartup` system, no `run_if`. `TerrainConfig` present at build() as before.
  Bind `terrain` to a local **before** the second `insert_resource` so the immutable `app.world()`
  borrow ends first (can't hold `&World` across `&mut app`). Also update `LevelPlugin`'s doc comment:
  it currently says "Loads the authored level (`PreStartup`)" — now build-time.
  Improvement worth noting: terrain now derives from the *same* `LevelSpec` a caller injects, so a
  test/override that pre-inserts a spec also drives terrain — strictly better than today's
  `level_terrain()`, which ignored any injected spec and always read the constants.
- `spawn.rs`: boundary walls read `level.map_half_extent` (not the deleted const). `MAP_HALF_EXTENT`
  import gone.
- `terrain.rs`: **deleted** (logic moves to `LevelSpec::terrain_config`). Remove `mod terrain;`.
- `constants.rs`: **deleted** — empties out. Remove `pub mod constants;` from `lib.rs`.

No new Cargo deps: `world` is already a `level` dependency, and `serde`/`ron` arrived in p27.

1. `manifest.rs`: add `RawTerrainSpec`/`RawFeaturePopulation`/`RawFeatureSpec` + the `to_population`/
   `to_feature` inherent mappers; extend `RawLevelSpec` with `map_half_extent` + `terrain`.
2. `spec.rs`: add `TerrainSpec`, extend `LevelSpec` (`map_half_extent`, `terrain`), the single legal
   `From<RawTerrainSpec>` impl (foreign fields built via the mappers — no per-type `From` on `world`
   types), extend `From<RawLevelSpec>`, add `terrain_config()`.
3. `loader.rs`: system → plain `fn` returning `LevelSpec`; drop bevy/`Commands`; fix doc comments.
4. `plugin.rs`: build-time `contains_resource` load + `terrain_config` insert; drop `PreStartup`/
   `run_if`; update the plugin doc comment.
5. `spawn.rs`: walls from `level.map_half_extent`; drop the `crate::constants::MAP_HALF_EXTENT` import
   and refresh the `MAP_HALF_EXTENT`-naming in its doc comment.
6. Update `objects.rs` module doc (`LevelSpec` is the whole-level root, not per-object).
7. Delete `terrain.rs` + `constants.rs`; prune `lib.rs` (`mod terrain;`, `mod constants;`). Confirmed:
   no other crate or test references the deleted symbols (only `spawn.rs`/`terrain.rs`/`plugin.rs` do).
8. Extend `assets/level.ron` with `map_half_extent` + `terrain` (values copied verbatim from the
   current consts — `0x5EED_1234`, hills 36, mountains 6, the two authored features).
9. `./bin/housekeeping.sh`; run the app to confirm identical terrain + world.

## Tests (decision needed — test edit)
- The integration test `tests/spawn_level.rs` inserts a `LevelSpec` **before `add_plugins`**
  (already does), so build()-time load skips the file. But the sample now needs `map_half_extent` +
  `terrain` fields. Forced *adaptation* (new required fields), assertions unchanged (still 4 walls /
  8 obstacles / 1 player). The `map_half_extent` value is arbitrary for these tests (they count
  walls, not positions), but the fields are now required to construct `LevelSpec`. **Needs your OK**
  under the no-silent-test-edits rule. Note this test now also exercises `terrain_config()` at
  build() — pre-inserting a `LevelSpec` without a valid `terrain` would panic, so the sample must
  carry one (small but real new coverage, not a workaround).
- `spec.rs` round-trip test extended to assert `map_half_extent`, a `FeaturePopulation` field, and an
  authored-feature `center` tuple→`Vec2`.
- New: `terrain_config()` maps `map_half_extent` → `TerrainConfig.half_extent` (the shared value).

## Out of scope
The elevation *engine* tuning (`FBM_*`, `FLAT_*`, contour spacing) stays in `world::elevation::
constants` — that's renderer/engine internals, not level-authored content. This moves only the
recipe `level` authors and owns.

<!-- auto-reviewed -->
