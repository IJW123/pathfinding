# Elevation & Contour Rendering Plan

## Context

The game is a small Bevy 0.18 2D pathfinding sandbox: a player, walls, collision detection. The world is currently flat and featureless. We're adding an **elevation system**: a continuous height field rendered as topographic contour lines, where the player's movement speed responds to terrain slope and a HUD shows the player's `(x, y, z)` position.

The system must scale to large maps from day one (chunked rendering, lazy spawn). The focus of this plan is **how the contour lines are drawn** — the data flow from height field → marching squares → GPU mesh → screen.

Decisions already locked in earlier conversation:
- 1 pixel = 1 meter
- Height range `[0, 100]`, contour every `10` units (~9 lines)
- Hand-rolled value-noise fBm, no new deps
- Marching grid: `10 px` per cell, `32×32` cells per chunk (= `320×320 px` per chunk)
- Slope only affects speed (no hard cliffs): `speed *= clamp(1 − k·dot(dir, ∇h), 0.2, 1.5)`
- HUD: top-right text, `x, y, z` with `z` sampled under the player
- Saddle ambiguity: "connect high corners"
- All elevation code lives under `src/world/elevation/` for now
- HUD lives under new `src/hud/` module

## How contour lines are drawn (the core mechanism)

### Data flow

```
HeightFn (analytic fBm)                              ← Resource, pure function
       │
       │  sampled on a 33×33 corner grid per chunk
       ▼
Per-chunk marching squares                           ← src/world/elevation/marching.rs
       │
       │  for each of ~9 iso levels, walks the 32×32 cell grid,
       │  emits 2 vertices per crossing segment
       ▼
Mesh (PrimitiveTopology::LineList)                   ← src/world/elevation/mesh_build.rs
       │  positions: Vec<[f32; 3]>
       │  colors:    Vec<[f32; 4]>   (per-vertex, color-coded by level)
       ▼
Entity: (Mesh2d, MeshMaterial2d<ColorMaterial>, Transform)
       │  spawned/despawned by streamer based on camera AABB
       ▼
GPU rasterizes as 1-px lines, blended via ColorMaterial(WHITE) × vertex color
```

No global grid is ever stored. The height field is point-evaluable from `HeightFn::sample(pos)`. Chunks only materialize a 33×33 corner sample buffer transiently during mesh build, then discard it. This is what makes the architecture map-size-agnostic.

### Marching squares — case table

Corner numbering: `0=BL, 1=BR, 2=TR, 3=TL`. Edge numbering: `0=bottom(0-1), 1=right(1-2), 2=top(3-2), 3=left(0-3)`. Case index: `(c0≥iso) + 2(c1≥iso) + 4(c2≥iso) + 8(c3≥iso)`. Saddle rule: "connect high corners".

```rust
// CASES[16]: list of edge-pairs (each pair = one line segment)
0:  []
1:  [(3,0)]
2:  [(0,1)]
3:  [(3,1)]
4:  [(1,2)]
5:  [(3,0), (1,2)]    // saddle, c0+c2 high
6:  [(0,2)]
7:  [(3,2)]
8:  [(2,3)]
9:  [(2,0)]
10: [(0,1), (2,3)]    // saddle, c1+c3 high
11: [(2,1)]
12: [(3,1)]
13: [(0,1)]
14: [(3,0)]
15: []
```

Per cell: compute 4 edge-crossing points once (only those edges that actually have a sign change). Edge crossing position is linear interp: `p = A + t(B − A)` where `t = (iso − a) / (b − a)` between corner values `a`, `b`. Then index `CASES[case]` and push 2 vertices per segment into the mesh buffers.

### Per-level coloring (free, ship in v1)

`ColorMaterial` automatically picks up `Mesh::ATTRIBUTE_COLOR` (Bevy enables `VERTEX_COLORS` shader def when the attribute is present — verified in `bevy_sprite_render-0.18.1/src/mesh2d/mesh.rs:525-527`). So each segment's 2 vertices get the same color, chosen from a palette indexed by level. Default palette: a gentle gradient from dark brown (low) to off-white (high). Single mesh per chunk, all levels collapsed in.

### Bevy 0.18 mesh assembly (the exact API)

```rust
use bevy::asset::RenderAssetUsages;
use bevy::mesh::PrimitiveTopology;

let mut mesh = Mesh::new(
    PrimitiveTopology::LineList,
    RenderAssetUsages::RENDER_WORLD,   // free CPU copy after upload
);
mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions); // Vec<[f32;3]>
mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR,    colors);    // Vec<[f32;4]>

commands.spawn((
    Mesh2d(meshes.add(mesh)),
    MeshMaterial2d(materials.add(ColorMaterial::from(Color::WHITE))),
    Transform::from_xyz(chunk_origin.x, chunk_origin.y, 0.1),
    ElevationChunk,
    ChunkCoord(coord),
));
```

`Mesh2dHandle` no longer exists in 0.18 — it's the tuple-component `Mesh2d(Handle<Mesh>)`. `Visibility` / `GlobalTransform` come in via required components, no need to spawn them.

### Z-ordering

- Walls: `z = 0.0` (existing)
- Contour chunks: `z = 0.1`
- Player: `z = 1.0` (existing)

### Chunk streaming

```rust
#[derive(Resource, Default)]
pub struct LoadedChunks(pub HashMap<IVec2, Entity>);
```

Each frame, the streamer system:
1. Reads camera `Transform` + `OrthographicProjection` → world-space AABB.
2. Expands by `CHUNK_VIEW_MARGIN` (200 px).
3. Computes the inclusive `IVec2` range of chunk coords intersecting that AABB.
4. For each coord not in `LoadedChunks` → build mesh synchronously, spawn entity, insert into map.
5. For each entry in `LoadedChunks` whose coord is outside the range → despawn, remove its mesh handle from `Assets<Mesh>` (since `RENDER_WORLD`-only meshes still hold a GPU slot via the handle).

Mesh build is synchronous in v1 (~30k ops per chunk, negligible). If profiling shows hitches later, move to `AsyncComputeTaskPool`.

## File layout

```
src/world/elevation/mod.rs           pub mod only
src/world/elevation/components.rs    ElevationChunk, ChunkCoord
src/world/elevation/resources.rs     HeightFn, LoadedChunks
src/world/elevation/height_fn.rs     fBm sample + gradient (central differences)
src/world/elevation/marching.rs      CASE_TABLE [u8; 16][...], edge_interp, contour_segments_for_cell
src/world/elevation/mesh_build.rs    build_chunk_mesh(coord, &HeightFn) -> Mesh
src/world/elevation/streaming.rs     update_visible_chunks system
src/world/elevation/slope.rs         slope_speed_multiplier(dir, gradient) -> f32
src/world/elevation/plugin.rs        ElevationPlugin

src/hud/mod.rs                       pub mod only
src/hud/components.rs                CoordReadout marker
src/hud/systems.rs                   spawn_hud, update_coord_text
src/hud/plugin.rs                    HudPlugin
```

Per CLAUDE.md: every `mod.rs` contains *only* `pub mod ...` declarations — no structs, traits, impls, or fns.

## Constants to add (src/constants.rs)

```rust
pub const PIXELS_PER_METER: f32 = 1.0;
pub const ELEVATION_CELL: f32 = 10.0;
pub const ELEV_CHUNK_CELLS: usize = 32;
pub const CONTOUR_STEP: f32 = 10.0;
pub const HEIGHT_MIN: f32 = 0.0;
pub const HEIGHT_MAX: f32 = 100.0;
pub const SLOPE_SPEED_K: f32 = 0.6;
pub const SLOPE_SPEED_MIN: f32 = 0.2;
pub const SLOPE_SPEED_MAX: f32 = 1.5;
pub const CHUNK_VIEW_MARGIN: f32 = 200.0;
pub const FBM_SEED: u32 = 0xC0FFEE;
pub const FBM_OCTAVES: u32 = 4;
pub const FBM_LACUNARITY: f32 = 2.0;
pub const FBM_GAIN: f32 = 0.5;
pub const FBM_BASE_FREQ: f32 = 1.0 / 200.0;
```

## HeightFn details

Value-noise fBm, hand-rolled, no deps:

- `hash2(ix, iy, seed) -> u32`: tiny integer hash (xorshift / wangs-hash style).
- `value_noise(p, seed) -> f32`: bilinear interp of 4 integer-corner hashes, smoothstep-warped.
- `fbm(p, seed) -> f32`: sum of octaves of `value_noise` at increasing frequencies, decreasing amplitudes; normalized to `[HEIGHT_MIN, HEIGHT_MAX]`.
- `HeightFn::sample(pos: Vec2) -> f32`: calls `fbm` once.
- `HeightFn::gradient(pos: Vec2) -> Vec2`: central differences with `ε = ELEVATION_CELL` (4 `sample()` calls).

`HeightFn` is a `Resource` carrying the seed/parameters; methods are deterministic given the seed.

## Slope effect — integration with existing player

`src/player/systems.rs::move_player` currently mutates `Transform` directly. Add `height: Res<HeightFn>` to the signature. Replace:

```rust
let delta = direction.normalize() * PLAYER_SPEED * time.delta_secs();
```

with:

```rust
let dir = direction.normalize();
for mut transform in &mut query {
    let grad = height.gradient(transform.translation.xy());
    let slope_mul = (1.0 - SLOPE_SPEED_K * dir.dot(grad))
        .clamp(SLOPE_SPEED_MIN, SLOPE_SPEED_MAX);
    let delta = dir * PLAYER_SPEED * slope_mul * time.delta_secs();
    transform.translation.x += delta.x;
    transform.translation.y += delta.y;
}
```

Helper `slope_speed_multiplier(dir, grad) -> f32` lives in `src/world/elevation/slope.rs` so the player system stays a one-liner.

No new system needed. Scheduling unchanged: `move_player.before(CollisionSet)`.

## HUD — top-right `(x, y, z)`

- `spawn_hud` (Startup): one `Node` anchored top-right with `position_type: Absolute`, `right: Val::Px(8.0)`, `top: Val::Px(8.0)`; child `Text` + `CoordReadout` marker.
- `update_coord_text` (Update, runs after `move_player`): query player `Transform`, sample `HeightFn`, write `format!("x: {:.1}  y: {:.1}  z: {:.1}", x, y, z)` into the text component.

Bevy 0.18 UI text uses `Text("...".into())` + `TextFont { ... }` + `TextColor(...)` as tuple components. Default font is enabled via `default_font` feature already in `Cargo.toml`.

## System ordering (final)

```
Startup:
  setup_camera, setup_player, spawn_bounds (existing)
  spawn_hud                                       (new)
  // HeightFn / LoadedChunks initialized via init_resource in ElevationPlugin

Update:
  update_visible_chunks                           (new — spawns/despawns chunks)
  move_player.before(CollisionSet)                (existing — now reads HeightFn)
  CollisionSet                                    (existing, unchanged)
  update_coord_text.after(move_player)            (new — HUD)
```

## Critical files to modify or create

**Modify:**
- `src/main.rs` — register `ElevationPlugin`, `HudPlugin`, declare `mod hud;`.
- `src/world/mod.rs` — add `pub mod elevation;`.
- `src/constants.rs` — append elevation constants.
- `src/player/systems.rs` — slope-modulate `move_player` velocity.

**Create:**
- All files in `src/world/elevation/` and `src/hud/` listed above.

## Verification

1. **Builds clean.** `cargo build` — no warnings. Run housekeeping per CLAUDE.md if applicable to this repo (CLAUDE.md references a pixelconnector script — confirm whether it applies here; if not, just `cargo clippy --all-targets -- -D warnings`).
2. **Run the game.** `cargo run`. Expect:
   - Topographic contour lines visible across the map, multiple bands, faintly colored per level.
   - Lines stay sharp when panning (chunks stream in/out invisibly).
   - HUD top-right shows live `x, y, z` while moving.
   - Walking "uphill" (against gradient) is noticeably slower; "downhill" faster. No teleports, no jitter.
3. **Edges of chunks line up.** Walk across a chunk boundary — contour lines should be continuous (they will be, because each chunk samples `HeightFn` at shared corner positions and the function is deterministic).
4. **Slope behaves at flat regions.** Stand in a flat spot (gradient ≈ 0) — speed should be ~`PLAYER_SPEED`.
5. **Clamp prevents stalls.** Walk straight into the steepest slope you can find — speed should not drop below `SLOPE_SPEED_MIN * PLAYER_SPEED`.
6. **Kill server after testing** (CLAUDE.md rule, even though there's no port 5000 here — for muscle memory, confirm `cargo run` is terminated cleanly).

## Out of scope (deliberate)

- Isoband fills (colored regions between contours). Easy follow-up: same case table → 16-case isoband table → `TriangleList`.
- Thick anti-aliased lines. Reversible swap to triangulated quads if v1 looks too thin.
- Async chunk mesh build. Trivial to retrofit if hitches appear.
- Hard cliffs / impassable steep terrain. Speed-only for now.
- Pathfinding cost-field integration. Field is already exposed via `HeightFn` for future use.
- Persistence / save-load of seed. Hardcoded `FBM_SEED` is fine for v1.
