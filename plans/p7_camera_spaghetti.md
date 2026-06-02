# Plan: Remove chunk streaming — render the map as static contour tiles

## Context

`update_loaded_chunks` (`crates/world/src/elevation/chunk_lifecycle.rs`) is a per-frame,
camera-driven loader: it reads the camera's visible rect, diffs the overlapping chunk set
against `LoadedChunks`, spawns/despawns chunk entities, and emits `ChunkLoaded`/`ChunkUnloaded`
that `render` consumes to build meshes.

This couples **world (simulation) to the render camera** — it queries `Single<(&Transform,
&Projection), With<Camera2d>>`, so world panics without a camera and lets view state drive
the sim. It's also **premature optimization**: built for an infinite world that since became
a fixed 4 km map. The data already loads fully at launch (`HeightField` is a baked 401×401
grid), `HeightField::sample` is now a cheap bilinear lookup (not fBm), and **Bevy already
frustum-culls offscreen `Mesh2d` entities** — so streaming buys nothing here. The
`ContourCache` only exists to paper over reload churn that this streaming creates.

**Goal:** delete streaming. World = pure data + functions, fully loaded at launch, zero
camera/render awareness. Render builds all contour tile meshes once at `Startup` as static
entities; Bevy culls offscreen ones. The fixed tile partition is retained (not for
streaming) so future localized terrain edits can re-extract only the touched tiles — but no
edit/dirty machinery is built now (YAGNI).

## Cost sanity check (why static-at-launch is fine)
4 km map, 320 m chunks → coords −7..6 per axis = **14×14 = 196 tiles**, built once. Each is a
33×33 grid lookup + marching squares; total a few ms at startup (cheaper than the
`HeightField` gen already done). Resident geometry is tens of thousands of line vertices —
trivial. Most of the map is flat → many tiles have zero contour segments and are **skipped**,
so the live entity count is well under 196.

## Delete (world crate)
- `elevation/chunk_lifecycle.rs` — the camera diff loop.
- `elevation/chunk_view.rs` — `desired_chunks` (camera-rect → set).
- `elevation/loaded_chunks.rs` — `LoadedChunks`.
- `elevation/chunk_events.rs` — `ChunkLoaded` / `ChunkUnloaded` (`ChunkUnloaded` already has
  no consumer).
- `elevation/components.rs::ElevationChunk` — the mesh marker moves to render (below); if
  that leaves `components.rs` empty, delete the file + its `mod` line.
- `elevation/constants.rs::CHUNK_VIEW_MARGIN`.
- Remove the deleted `mod` lines from `elevation/mod.rs`.

## Delete (render crate)
- `elevation/contour/cache.rs` — `ContourCache` (no reload → no cache).

## Keep (world crate — pure data + functions render imports)
`height_field.rs`, `chunk_coord.rs`, `contour/{data,extract,marching}`, `noise`,
`generation`, the remaining `elevation/constants.rs` (`ELEVATION_CELL`, `ELEV_CHUNK_CELLS`,
`HEIGHT_*`, `FBM_*`, generation consts).

## Add (world crate) — `chunk_coord.rs`
A pure enumerator of every tile coord covering the map (replaces `desired_chunks`):
```rust
#[must_use]
pub fn map_chunk_coords() -> Vec<IVec2> {
    let span = ELEV_CHUNK_CELLS as f32 * ELEVATION_CELL;          // 320 m
    let min = (-MAP_HALF_EXTENT / span).floor() as i32;            // -7
    let max = ( MAP_HALF_EXTENT / span).floor() as i32;            //  6
    (min..=max)
        .flat_map(|cy| (min..=max).map(move |cx| IVec2::new(cx, cy)))
        .collect()
}
```
(Needs `MAP_HALF_EXTENT` from `crate::constants`.)

## Rewrite (world crate) — `elevation/plugin.rs`
Drop `ElevationLifecycleSet`, the `LoadedChunks` resource, both messages, and the streaming
system. `ElevationPlugin::build` shrinks to:
```rust
app.init_resource::<HeightField>();
```
(`init_resource` runs `Default` at plugin-build time, so the grid is populated before any
`Startup` system reads it — no ordering set needed.)

## Rewrite (render crate)
- New `elevation/components.rs`: `#[derive(Component)] pub struct ContourTile;` (moved/renamed
  from world's `ElevationChunk`; identifies terrain entities, no coord map — that's edit infra,
  deferred). Add `pub mod components;` to render's `elevation/mod.rs`.
- `elevation/contour/render.rs`: replace `render_contours_on_chunk_loaded` (Update, reacts to
  `ChunkLoaded`) with a `Startup` system:
  ```rust
  pub fn spawn_contour_tiles(
      mut commands: Commands,
      mut meshes: ResMut<Assets<Mesh>>,
      mut materials: ResMut<Assets<ColorMaterial>>,
      height: Res<HeightField>,
      levels: Res<ContourLevels>,
      style: Res<ContourStyle>,
  ) {
      let material = materials.add(ColorMaterial::from(Color::WHITE)); // one, shared
      for coord in map_chunk_coords() {
          let lines = extract_contours(coord, &height, &levels.0);
          if lines.iter().all(|l| l.segments.is_empty()) { continue; } // skip flat tiles
          let origin = chunk_origin_world(coord);
          commands.spawn((
              Transform::from_xyz(origin.x, origin.y, 0.1),
              Mesh2d(meshes.add(contour_lines_to_mesh(&lines, &style))),
              MeshMaterial2d(material.clone()),
              ContourTile,
          ));
      }
  }
  ```
- `elevation/plugin.rs` (render): drop the `ElevationLifecycleSet` import + `ContourCache`
  init; register `add_systems(Startup, spawn_contour_tiles)`. Keep `ContourLevels` /
  `ContourStyle` init.

## Verification
- `grep -rn "Camera2d\|Projection\|PrimaryWindow" crates/world/src` → **no matches**. World is
  view-agnostic and has no streaming.
- `cargo build` clean; `cargo clippy --workspace` zero warnings.
- `cargo run -p pathfinding`: terrain renders identically at launch (contour rings on
  hills/mountains, empty flats); pan/zoom show no empty edges and no pop-in (everything's
  already there); zooming in still culls offscreen tiles (Bevy frustum culling). No camera =
  world still initialises (e.g. a future headless run builds `HeightField` fine).

## Knock-on
- Obsoletes `p8`'s item #1 (per-frame chunk-load budget) — there's no streaming to budget.
  Trim it from `p8` when that plan is executed.
- Future alterable terrain: mutate the `HeightField` grid, mark the overlapping `ContourTile`
  coords dirty, re-extract only those meshes. The tile partition kept here is exactly that
  hook — not built now.
