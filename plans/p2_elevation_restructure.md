# P2 — Elevation module restructure

Goal: make `world/elevation/` scale as more systems (pathfinding, physics,
AI, projectiles) start consuming it, without forcing them to drag in
rendering or movement concerns.

Each step is independent and can land separately.

---

## Step 0 — DONE: extract slope into `world/terrain_effects/`

- `src/world/terrain_effects/slope_speed.rs` owns the formula + its constants.
- `SLOPE_SPEED_K/MIN/MAX` removed from `src/constants.rs`.
- `src/world/elevation/slope.rs` deleted.
- Player import updated.

Pattern established: each terrain-effect formula gets its own file with
co-located tuning constants. Future siblings: `friction.rs`, `visibility.rs`,
`drag.rs`, etc.

---

## Step 1 — Split `streaming.rs`: chunk lifecycle vs. contour rendering

**Problem.** `update_visible_chunks` mixes six responsibilities:
viewport math, desired-set computation, contour extraction, mesh build,
material creation, entity spawn/despawn. When pathfinding or physics need
the same chunks, they cannot depend on `ColorMaterial`.

**Target layout.**

```
src/world/elevation/
  chunk_lifecycle.rs   // load/unload only, emits events
  chunk_events.rs      // ChunkLoaded { coord }, ChunkUnloaded { coord }
  chunk_view.rs        // camera viewport -> set of desired IVec2 coords
```

```
src/world/elevation/contour/
  render.rs            // reacts to ChunkLoaded, builds + spawns mesh entity;
                       // reacts to ChunkUnloaded, despawns mesh entity.
```

**Responsibilities after the split:**
- `chunk_view::desired_chunks(cam_pos, viewport_size) -> HashSet<IVec2>`
  — pure function, no Bevy world access.
- `chunk_lifecycle::update_loaded_chunks` system — diffs desired vs
  `LoadedChunks`, fires `ChunkLoaded` / `ChunkUnloaded` events. Owns the
  `LoadedChunks` resource (which now stores just the chunk entity, no mesh).
- `contour::render::on_chunk_loaded` — receives `ChunkLoaded`, runs
  `extract_contours` + `contour_lines_to_mesh`, spawns mesh + material as
  a child of the chunk entity (or a sibling tagged `ContourRender`).
- Future systems (pathfinding bake, physics colliders) attach themselves
  the same way: `on_chunk_loaded` -> do their thing -> attach component.

**Why events, not direct spawn calls.**
Decouples producers from consumers. Adding a new consumer (e.g.
`pathfinding::bake_costs_on_chunk_loaded`) does not touch elevation.

**Notes.**
- Keep the `ElevationChunk` component as the canonical chunk entity.
- `ContourGeometry` component stays where it is (on the chunk entity),
  populated by the render system, not by streaming.
- The `Update` schedule ordering: `update_loaded_chunks` must run before
  any `on_chunk_loaded` consumer in the same frame, otherwise the event
  is delayed one tick. Use `.before()` or a system set.

---

## Step 2 — Split `height_fn.rs`: resource vs. noise primitives

**Problem.** ~50 lines of fbm/value_noise/hash live in the same file as
the `HeightFn` resource. Cannot swap noise backend without touching the
resource definition, and other systems cannot reuse the noise without
pulling in a Bevy `Resource`.

**Target layout.**

```
src/world/elevation/
  height_fn.rs         // HeightFn resource only (façade over noise)
  noise/
    mod.rs             // declarations only
    fbm.rs             // fbm() + FBM tuning constants (FBM_OCTAVES etc.)
    value_noise.rs     // value_noise(), hash2, hash_unit, smoothstep
```

**Migration.**
- `FBM_SEED`, `FBM_OCTAVES`, `FBM_LACUNARITY`, `FBM_GAIN`, `FBM_BASE_FREQ`
  move from `src/constants.rs` to `noise/fbm.rs`.
- `HeightFn` keeps its public API (`sample`, `gradient`); internals call
  `noise::fbm::fbm(...)`.
- `HEIGHT_MIN/MAX` and `ELEVATION_CELL` stay in `src/constants.rs` for now
  — they cross module boundaries (used by `chunk_coord`, `contour_extract`,
  `contour_levels`).

**Future option.** If we later want a different sampling strategy (GPU,
lookup table, layered noise), `HeightFn` becomes a trait or enum and the
file structure already supports it.

---

## Step 3 — Normalize resource file naming

**Problem.** Inconsistent: `LoadedChunks` in `resources.rs`, but every other
resource has its own file (`height_fn.rs`, `contour_levels.rs`,
`contour_style.rs`).

**Fix.** Rename `resources.rs` -> `loaded_chunks.rs`. Update one import
in `streaming.rs` (or `chunk_lifecycle.rs` after Step 1).

Trivial, low risk. Do this alongside Step 1 since `streaming.rs` is being
rewritten anyway.

---

## Step 4 — Group `contour_*` into a submodule

**Problem.** Five `contour_*.rs` files dangle at the top of `elevation/`.
`marching.rs` is only used by contour extraction, but its generic name
hides the coupling.

**Target layout.**

```
src/world/elevation/contour/
  mod.rs               // declarations only
  data.rs              // ContourSegment, ContourLine (was contour_data.rs)
  extract.rs           // extract_contours (was contour_extract.rs)
  levels.rs            // ContourLevels resource (was contour_levels.rs)
  marching.rs          // CASE_TABLE, emit_cell_segments (was top-level)
  mesh.rs              // contour_lines_to_mesh (was contour_mesh.rs)
  render.rs            // NEW from Step 1
  style.rs             // ContourStyle resource (was contour_style.rs)
```

**Notes.**
- `CONTOUR_STEP` moves from `src/constants.rs` to `contour/levels.rs`
  (only used there).
- Update `ElevationPlugin` imports.
- Pure renames + path updates, no logic change.

Do this after Step 1 so we move `render.rs` into its final home in one go.

---

## Step 5 — Constants policy cleanup

After Steps 0–4, audit `src/constants.rs`. Expected end state:

**Stays in `src/constants.rs`** (cross-module):
- `CELL_SIZE`, `WALL_THICKNESS` — used by world/wall + collision.
- `PLAYER_SPEED`, `PLAYER_SIZE` — used by player.
- `HEIGHT_MIN`, `HEIGHT_MAX`, `ELEVATION_CELL`, `ELEV_CHUNK_CELLS`,
  `CHUNK_VIEW_MARGIN` — used across multiple elevation files.

**Moves into module-local files:**
- `SLOPE_SPEED_*` — DONE, in `terrain_effects/slope_speed.rs`.
- `FBM_*` — into `elevation/noise/fbm.rs` (Step 2).
- `CONTOUR_STEP` — into `elevation/contour/levels.rs` (Step 4).

**Rule going forward.** If exactly one file reads a constant, it lives in
that file. If a single module reads it from two+ files, it lives in a
sibling `constants.rs` for that module. Only put it in
`src/constants.rs` if it crosses module boundaries.

---

## Suggested execution order

1. Step 3 (rename `resources.rs`) — 2 minutes, no risk, clears noise.
2. Step 1 (split streaming) — biggest scalability win, do this next.
3. Step 4 (contour submodule) — pairs naturally with Step 1's render.rs.
4. Step 2 (split noise out of height_fn) — independent, can land anytime.
5. Step 5 (constants audit) — falls out of the above; final pass.

Each step ends with `cargo check` clean and the game still running.
