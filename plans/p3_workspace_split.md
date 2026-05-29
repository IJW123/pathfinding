# P3 — Cargo workspace split: gameplay vs. rendering

Continuation of P2's "Open follow-up: rendering / gameplay separation."
Decision: convert the single binary crate into a Cargo workspace with
strict separation between gameplay/sim crates and the render crate.

---

## Target layout

```
pathfinding/
  Cargo.toml          # [workspace] + [workspace.dependencies]
  crates/
    app/              # bin: main.rs + plugin wiring + camera setup
    collision/        # lib: leaf (no game-side deps)
    world/            # lib: gameplay/sim (elevation, walls, terrain effects)
    player/           # lib: input + movement
    hud/              # lib: text overlay (intrinsically visual)
    render/           # lib: all sprite/mesh/material attachment
```

## Dep DAG

```
app
 ├─ render  → world, player
 ├─ hud     → world, player
 ├─ player  → world, collision
 ├─ world   → collision
 └─ collision
```

`render` is always downstream of gameplay; the world/player crates
have zero `bevy::sprite` / `bevy::render` imports after the split.

## Naming

Unprefixed crate names (`world`, `render`, etc.). Workspace is internal,
no crates.io collision risk.

## Bevy features (Phase 1)

Move `bevy = { ... }` to `[workspace.dependencies]` with current full
feature set. Every crate inherits the same set via
`bevy = { workspace = true }`. Tightening per-crate features (no
`bevy_render` in gameplay crates) is a separate later pass.

---

## Step-by-step execution

### Step 1 — Workspace skeleton

- Root `Cargo.toml`: replace `[package]` with `[workspace]` listing
  `crates/*` as members. Move bevy dep into `[workspace.dependencies]`.
- Create `crates/{app,collision,world,player,hud,render}/` with stub
  `Cargo.toml` and empty `src/lib.rs` (or `src/main.rs` for app).
- `cargo check` clean.

### Step 2 — Move files (no logic changes yet)

For each existing module, move its `.rs` files into the matching crate's
`src/` directory and create a `lib.rs` with the same `pub mod`
declarations the old `mod.rs` had.

- `src/collision/*` → `crates/collision/src/`
- `src/world/*` (excluding the files moving to render) →
  `crates/world/src/`
- `src/player/*` → `crates/player/src/`
- `src/hud/*` → `crates/hud/src/`
- `src/main.rs` → `crates/app/src/main.rs`

Render-bound files (Step 3 below):
- `src/world/elevation/contour/render.rs` → `crates/render/src/elevation/contour/render.rs`
- `src/world/elevation/contour/mesh.rs` → ditto
- `src/world/elevation/contour/style.rs` → ditto
- `src/world/elevation/contour/levels.rs` → ditto (only render reads it)
- `ContourGeometry` component → `crates/render/src/elevation/components.rs`

Update every `use crate::...` that crosses a crate boundary to
`use <crate>::...`. Intra-crate paths stay as `crate::...`.

Each lib's `lib.rs` is declarations only (CLAUDE.md: no structs/impls/
fns in `mod.rs` — same rule for `lib.rs`).

### Step 3 — Strict render separation: wall + player sprites

- `world/wall.rs`: drop `Sprite`, spawn `Wall + Collider + Solid + Static
  + Transform` only. No `bevy::sprite` import.
- `render/wall.rs` (new): system reacting to `Added<Wall>`, attaches
  `Sprite` with the wall color + `custom_size` derived from `Collider`.
  Wired via a new `WallRenderPlugin`.
- `player/systems.rs::setup_player`: drop `Sprite`, spawn `Player +
  Collider + Transform` only.
- `render/player.rs` (new): system reacting to `Added<Player>`,
  attaches `Sprite`. Wired via `PlayerRenderPlugin`.

Open detail during impl: where does the wall color come from? Today
it's a local `Color::srgb(0.4, 0.4, 0.4)` in `spawn_bounds`. Move it
into `render/wall.rs` as a local constant, or into `render/constants.rs`
if it grows. Same question for player.

### Step 4 — Fix lifecycle weakness #2 (`.chain()`)

- Define a `SystemSet` in `world::elevation::plugin`:
  `pub struct ElevationLifecycleSet;`
- `update_loaded_chunks` goes into `ElevationLifecycleSet`.
- `ContourRenderPlugin` schedules `render_contours_on_chunk_loaded`
  with `.after(ElevationLifecycleSet)`. No `.chain()`.
- Future `ChunkLoaded` consumers do the same — parallel with each
  other, after lifecycle.

This closes weakness #2 in `project_elevation_lifecycle_weaknesses`.
Weaknesses #1 (readiness) and #3 (`ChunkUnloaded` ordering) stay
deferred until they bite.

### Step 5 — Plugin wiring in app

`app/main.rs` adds:
- `CollisionPlugin`
- `WorldPlugin` (walls)
- `ElevationPlugin` (chunk lifecycle, HeightFn)
- `PlayerPlugin` (input + movement)
- `HudPlugin`
- `ContourRenderPlugin` (new in render)
- `WallRenderPlugin` (new in render)
- `PlayerRenderPlugin` (new in render)

Or a single `RenderPlugin` that bundles the three — TBD during impl.
Preference: bundle, since "add render" is the user-facing toggle.

### Step 6 — Verify

- `cargo check` clean across workspace.
- `cargo run` — visually identical to pre-split. Walls visible, player
  visible, contours visible, movement unchanged.

---

## Files that move (concrete checklist)

**To `crates/collision/src/`:**
components.rs, constants.rs, events.rs, narrow_phase.rs, plugin.rs,
response.rs, spatial_hash.rs

**To `crates/world/src/`:**
components.rs, constants.rs, plugin.rs, wall.rs (stripped of Sprite),
terrain_effects/{mod-as-lib-section, constants.rs, slope_speed.rs},
elevation/{chunk_coord, chunk_events, chunk_lifecycle, chunk_view,
components (minus ContourGeometry), constants, height_fn,
loaded_chunks, plugin}.rs,
elevation/contour/{data, extract, marching}.rs,
elevation/noise/{fbm, value_noise}.rs

**To `crates/player/src/`:**
components.rs, constants.rs, plugin.rs, systems.rs (stripped of Sprite)

**To `crates/hud/src/`:**
components.rs, plugin.rs, systems.rs

**To `crates/render/src/`:**
elevation/contour/{render, mesh, style, levels}.rs (from world),
elevation/components.rs (ContourGeometry only),
wall.rs (new), player.rs (new), plugin.rs (new, bundles render plugins)

**To `crates/app/src/`:**
main.rs

---

## Risks / things to watch

- **mod.rs → lib.rs translation**: existing `mod.rs` files were
  declarations only. Same rule applies to `lib.rs`. Don't smuggle code in.
- **`use crate::` rewrites**: easy to miss one. After moves, grep each
  crate's `src/` for stale `crate::collision`/`crate::world` paths that
  should be `collision::`/`world::`.
- **Constants split across crates**: `WALL_THICKNESS` lives in
  `world::constants` but render needs sprite sizing matched to it. Sprite
  size derives from `Collider::half_extents` (already on the entity), so
  render doesn't need to import `WALL_THICKNESS` directly. Good.
- **`Static` component**: defined in collision, used by world::wall. Cross
  reference is fine since world → collision.
- **Bevy plugin order**: `Added<Wall>` reactions in render must run after
  `spawn_bounds` in world::plugin in the same frame, or the wall appears
  one frame late. Use `.chain()` or `.after()` at the app level, OR rely
  on Bevy's default Update ordering being a single frame (Added queries
  catch entities added the same tick if the system runs after). Verify
  during impl.

---

## STATUS — 2026-05-29: shipped, pending visual verification

All six steps landed. `cargo check` clean, `cargo build` clean,
`timeout 5 cargo run` no panic. Workspace structure matches plan.
Lifecycle weakness #2 closed (ElevationLifecycleSet in world,
`.after(ElevationLifecycleSet)` in render).

Sprite separation confirmed: `crates/{world,player,collision}/src`
contain zero `Sprite` / `bevy::sprite` references. All sprite
attachment lives in `crates/render/src/{wall,player}/plugin.rs`
behind `Added<Wall>` / `Added<Player>` queries.

**User must visually verify:** walls visible, player visible, contours
visible, movement unchanged. Watch for one-frame Sprite latency on
walls/player from the `Added<>` reaction (should be invisible since
sprites attach on the first Update tick after Startup spawn).

## Out of scope (do NOT do in P3)

- Per-crate Bevy feature trimming (Phase 2, later).
- Fixing lifecycle weaknesses #1 (LoadedChunks readiness) and #3
  (ChunkUnloaded ordering) — defer until a second ChunkLoaded consumer
  arrives.
- Splitting hud into "hud state" vs "hud render" — hud is the
  visualization for player coord, stays unified.
- Touching collision/player/world internal architecture beyond the
  Sprite extraction in Step 3.
