# Plan: Stored heightmap — flat base + stamped hills/mountains

## Context

Elevation was a pure analytic function: `HeightFn::sample(pos)` ran 4-octave fBm over the
whole plane, so the world was **rolling hills everywhere with no flat ground**. The desired
terrain is **mostly flat with occasional natural-looking hills and mountains** between the
flats.

Decision: move to a **stored 2D heightmap matrix**, filled by two generators — a *flat base*
(mostly flat, slight undulation so it isn't dead-level) and a *hill/mountain stamper* that
adds smooth, slightly-noisy elevation features. Features are placed **both** ways: a
hand-authored list plus seeded-random placement.

The heightmap is **immutable after generation**, so the per-chunk contour cache stays valid,
and the public field contract (`Resource` + `sample`/`gradient`) is preserved so every
downstream consumer only changes an import path.

## Public contract (unchanged behaviour for consumers)

Consumers need exactly: a `Resource`, `sample(Vec2) -> f32`, `gradient(Vec2) -> Vec2`.
- `move_player` (`crates/player/src/systems.rs`) → `gradient`
- `update_hud_text` (`crates/hud/src/systems.rs`) → `sample`
- `extract_contours` (`crates/world/src/elevation/contour/extract.rs`) → `sample`

The change is contained to the `world` elevation module plus mechanical
`HeightFn` → `HeightField` import/type renames in those files + the render system
(`crates/render/src/elevation/contour/render.rs`) and `elevation/plugin.rs`.

## Design

### Grid (stored matrix)
Anchored at world origin, spacing `ELEVATION_CELL` (10), covering the finite map:
`dims = (2 * MAP_HALF_EXTENT / ELEVATION_CELL) + 1` → 401×401 ≈ 161k f32 ≈ 643 KB.
Grid nodes coincide with the contour sampler's positions (chunk origin + i·10), so contour
extraction reads nodes with no interpolation error; the player/HUD interpolate.

### New: `crates/world/src/elevation/height_field.rs`
```rust
#[derive(Resource)]               // drop Copy/Clone (holds a Vec)
pub struct HeightField {
    dims: UVec2,
    origin: Vec2,                 // -MAP_HALF_EXTENT
    cell: f32,                    // ELEVATION_CELL
    data: Vec<f32>,               // row-major
}
```
- `sample(pos)`: bilinear interp, edge-clamped (nothing exists outside the map).
- `gradient(pos)`: central differences via `sample` (same approach as today, eps = cell).
- `impl Default`: build the grid once — for each node `p`,
  `clamp(flat_base(p) + Σ hill_value(p, h) for h in features, HEIGHT_MIN, HEIGHT_MAX)`.
  `init_resource::<HeightField>()` triggers this at startup. `hill_value` early-outs to 0
  beyond a feature's radius (no fBm cost for distant features), keeping startup cheap.

### New: `crates/world/src/elevation/generation/` (no items in mod.rs)
- `hill.rs`:
  - `pub struct HillSpec { center: Vec2, radius: f32, height: f32 }`
  - `flat_base(p)` = `FLAT_AMP * fbm(p * FLAT_FREQ, …)` — gentle low undulation; reuses
    existing `fbm` (`elevation/noise/fbm.rs`).
  - `hill_value(p, &HillSpec)` = `height * (1 - smoothstep(t)) * detail`, where
    `t = clamp(dist/radius, 0, 1)` and `detail = 1 + HILL_ROUGHNESS*(fbm(p*HILL_DETAIL_FREQ)-0.5)`
    to break perfect-circle domes. Returns 0 when `t >= 1`.
- `placement.rs`:
  - `pub const AUTHORED_HILLS: &[HillSpec]` — hand-placed features (a couple of examples).
  - `random_features(seed, count, radius_range, height_range)` → `Vec<HillSpec>` via
    deterministic hashing (no `rand` dependency).
  - `all_features()` → authored ++ seeded random **hills** ++ seeded random **mountains**
    (two passes with separate count/size/height ranges, so "mostly flat with a few big
    mountains" is tunable).

### Reuse hashing (avoid duplication)
Extract the integer hash currently inlined in `elevation/noise/value_noise.rs` into a new
`elevation/noise/hash.rs` (`pub fn hash_u32`, `pub fn hash_to_unit`); have both
`value_noise.rs` and `placement.rs` use it. Add `pub mod hash;` to `noise/mod.rs`.

### Constants (`crates/world/src/elevation/constants.rs`)
Add: `FLAT_AMP`, `FLAT_FREQ`, `HILL_ROUGHNESS`, `HILL_DETAIL_FREQ`, `FEATURE_SEED`,
hill pass (`HILL_COUNT`, `HILL_RADIUS_MIN/MAX`, `HILL_HEIGHT_MIN/MAX`), mountain pass
(`MOUNTAIN_COUNT`, `MOUNTAIN_RADIUS_MIN/MAX`, `MOUNTAIN_HEIGHT_MIN/MAX`). Existing fBm
params reused for base/detail noise.

### Module wiring & rename
- `elevation/mod.rs`: replace `pub mod height_fn;` with `pub mod height_field;`; add
  `pub mod generation;`. Delete `height_fn.rs`.
- `elevation/plugin.rs`: `init_resource::<HeightFn>()` → `HeightField`; update import.
- Rename `HeightFn` → `HeightField` (import + type) in: `extract.rs`, `render.rs`,
  `player/src/systems.rs`, `hud/src/systems.rs`.

## Consequences to expect
- **Plains render empty.** Flat base (~0–few units) sits below the first contour level (10),
  so flat areas show no contour lines; rings appear only on hills/mountains. Intended
  topographic look. A visible ground plane, if wanted later, is a separate render concern.
- Player moves at full speed on flats (gradient ≈ 0), slows climbing hills (existing
  slope-speed behaviour, now meaningful since most ground is flat).
- Contour `ContourLevels` auto-derive from `HEIGHT_MIN/MAX`; raise `HEIGHT_MAX` for more
  mountain rings (tuning, not structural).

## Verification
- `cargo build` clean; `cargo clippy --workspace` zero warnings.
- `cargo run -p pathfinding`: large flat areas with no contour lines, a handful of
  concentric-ring hills and taller mountains scattered "here and there", smooth edges. Pan
  around (WASD) — features stable, contours reattach instantly from cache. Drive the player
  (arrows) onto a hill — it slows on the climb. Kill the app after.
- Determinism: re-running produces the same map (seeded). Editing `AUTHORED_HILLS` adds a
  feature at the specified spot.

## Out of scope (tracked in TODO.md)
- Gating `update_loaded_chunks` on camera movement (#2).
- Terrain-type analytic field / gameplay traversal (#4).
