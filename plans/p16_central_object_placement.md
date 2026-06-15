# Centralize world object placement (`level` crate)

## Context

Today gameplay objects are spawned in three scattered `Startup` systems, each in a
different crate:

- `crates/obstacles/src/spawn.rs::setup_obstacles` — circle, triangle (static) + quad, pentagon (pushable)
- `crates/world/src/wall.rs::spawn_bounds` — 4 boundary walls
- `crates/player/src/systems.rs::setup_player` — player

The user wants **one place to insert objects into the world** so it stops being a scattered
mess — *while* leaving room for each object kind to grow into its own crate over time, and for
cross-cutting behaviours (e.g. NPC AI) to live in their own crates.

The resolution is to separate two things currently tangled together:

- **What an object *is*** (its components + behaviour) → grows complex → lives in a *kind crate*.
- **Where instances *go*** (placement/level layout) → the thing to centralize → a *level crate*.

This is the structure the user is reaching for. It's not a macro problem — Bevy 0.18's
**Required Components** (`#[require(...)]`) carry the fixed per-kind attributes, and small
`impl Bundle` constructor fns carry the parameterized ones. A central `level` crate holds the
placements. No macro, no Bevy Scenes (see "Explicitly not doing").

## Target structure (3 layers)

```
component / behaviour crates   hitboxes_rapier, collision_rapier, motion, (future) npc_ai
        ▲                      define shared components + systems; object-agnostic
        │
object-kind crates             obstacles (props), world (terrain+walls), player, (future) tree, building, npc
        ▲                      each exports: marker (+#[require]), constructor fn(s), behaviour Plugin
        │
assembly crates (siblings)     level  ── the ONE place: instance placement only
                               render ── attaches visuals via Added<Marker> (unchanged)
        ▲
       app                     wires plugins
```

Rules that keep it acyclic:
- **Markers + constructors stay in their kind crate.** `render` already depends on
  `obstacles::Obstacle` / `world::Wall` / `player::Player`; `level` depends on the same kinds.
  `level` and `render` are siblings, both depending *down* — no cycle.
- `level` never depends on `render` and vice versa. Render reacts to `Added<Marker>` regardless
  of who spawned the entity.

How this answers the growth question:
- A new **Tree** starts as a marker + `tree(pos, …)` constructor in a shared props crate
  (reuse `obstacles`, rename later if desired). When it needs behaviour, it gets its own crate +
  `TreePlugin`. `level` just gains a dep + placements; its shape never changes.
- **NPC AI** that spans kinds becomes a behaviour crate operating on components, sitting in the
  bottom layer next to `motion`/`collision_rapier`.

## Changes

### 1. Required Components for fixed attributes
`hitboxes_rapier/src/components.rs`: add `Default` to the flag markers so bare `#[require]` works
(verified: `#[require(Solid)]` expands to `<Solid as Default>::default`).
```rust
#[derive(Component, Default)] pub struct Solid;
#[derive(Component, Default)] pub struct Static;
```
Then on the kind markers:
- `Wall`   → `#[require(Solid, Static)]`
- `Obstacle` → `#[require(Solid)]`  *(NOT Static — quad/pentagon are pushable; Static stays a spawn arg)*
- `Player` → `#[require(Solid, MeasuredVelocity)]`  *(MeasuredVelocity already has Default)*

Boundary of the tool: `#[require]` only covers components identical for every instance.
`Collider` (per-shape), `Transform` (per-position), `Static` on obstacles (per-instance), and
`PrevPosition(spawn)` (parameterized by spawn pos — `motion`'s own doc warns of a first-frame
velocity spike if not seeded to spawn) are **constructor args**, never required.

### 2. Constructor fns in the kind crates (return `impl Bundle`)
Keep these with the kind so they're reusable for tests + future procedural placement. Take a
pre-built `Collider` so the fallible `Collider::convex(..) -> Result` stays at the call site.
- `obstacles`: `static_obstacle(transform, collider) -> impl Bundle` (adds `Obstacle, Static`),
  `pushable_obstacle(transform, collider) -> impl Bundle` (adds `Obstacle`).
- `world`: `wall(center: Vec2, half_extents: Vec2) -> impl Bundle`, plus
  `boundary_walls(half_extent, thickness) -> [impl Bundle; 4]` so positions are *derived* from
  map size, not four magic constants.
- `player`: `player(spawn: Vec2) -> impl Bundle` (adds `Player, Collider::obb, PrevPosition(spawn)`;
  `Solid`+`MeasuredVelocity` come from `#[require]`).

### 3. New `level` crate — the one place
- `crates/level/` — workspace member; deps: `bevy`, `obstacles`, `world`, `player`, `hitboxes_rapier`.
- `level/src/plugin.rs::LevelPlugin` registers one `Startup` system `spawn_level`.
- `level/src/spawn.rs::spawn_level` builds each `Collider` and calls the constructors as an honest
  **sequence of spawn calls** — `commands.spawn(static_obstacle(t, c))` etc. This is centralization
  (the one place), *not* a data table: a uniform table would need a closed `PlacementKind` enum that
  knows every kind, which fights the open-growth goal (see "Scaling placement" + "Deferred").

**Why a crate, not a module in `app`:** `app` is a `[[bin]]` — nothing can depend on it. Placement
must be a library to be testable or reusable in isolation (integration test, future second binary,
editor, headless sim). That is the present, concrete justification — not the deferred RON path.

**Smell guard:** `level` stays placement-only. `app` keeps *all* wiring — `DefaultPlugins`, the plugin
tuple, `configure_sets`, schedule ordering. The day `level` calls `configure_sets` or orders systems,
it has become `app #2` and the split has failed. `level` = content; `app` = wiring.

**Render decoupling is safe** because every spawn is `Startup`: render's `Added<Marker>` reaction sees
the entity the same frame, so the one-frame `Added` deferral never races. (Render could later move to
`on_add` observers — explicitly out of scope here.)

- `app/src/main.rs`: add `LevelPlugin`; **delete `ObstaclesPlugin` and `WorldPlugin`** (both only
  registered a spawn system — once placement moves out they build nothing; drop both from the plugin
  tuple); strip `setup_player`'s spawn from `PlayerPlugin` (keep `move_player` + `ElevationPlugin` —
  `player` depends on `world::HeightField`).

### 3b. Scaling placement (curated → generated → authored)
The constructors are the **stable spawn API**; placement has *feeders* that all funnel through them.
"The one place" is durable because adding scale swaps the feeder, not the API:
- **Feeder 1 — curated (now):** the const sequence in `spawn_level`. Handful of named set pieces.
- **Feeder 2 — generated (deferred, the real "thousands" path):** a system samples positions over the
  heightfield from a seed + density and calls the same constructors in a loop. Stores *rules, not rows* —
  thousands of obstacles never become thousands of file entries.
- **Feeder 3 — authored (deferred):** RON `LevelSpec` + `bevy_asset` loader for a hand-placed set.

Storage boundary: text/RON for authored immutable content; generation for bulk; **SQLite only if
save-games / mutable persisted state ever appear** — never for read-only layout (it adds a query
engine to load a list you read wholesale anyway).

### 4. Constants split (by ownership)
- **Stay with kind** (intrinsic/template): hull point clouds (`TRIANGLE_POINTS`, `QUAD_POINTS`,
  `PENTAGON_POINTS`), `CIRCLE_RADIUS`, `PLAYER_SIZE`, `WALL_THICKNESS`, `OBSTACLE_Z` (render-order
  policy), `MAP_HALF_EXTENT` (world).
- **Move to `level`** (instance): `CIRCLE_CENTER`, `TRIANGLE_CENTER`, `TRIANGLE_TILT_RADIANS`,
  `QUAD_CENTER`, `PENTAGON_CENTER`, player spawn.
- Rule of thumb: *if procedural placement would generate it, it's instance data (level); if it'd
  be read as a template, it's kind data.*

## Migration order (compiles + runs at each step)
1. `Default` on `Solid`/`Static`; add `#[require]` to markers; drop the now-redundant explicit
   `Solid` from the three spawners. Run — validates `require` + render's `Added<>` timing in place.
2. Add the constructor fns; existing Startup systems call them. Still scattered, now routed + testable.
3. Create empty `level` crate + `LevelPlugin`, wire into `app`. Compile.
4. Cut over one kind at a time, each atomic: move its placements + instance consts into `level`,
   delete its spawn system in the same commit (obstacles → walls → player). Run after each.
5. Tidy: **delete `ObstaclesPlugin` and `WorldPlugin`** (both become empty once their spawn system
   moves — drop from `app`'s tuple, don't "slim"); `PlayerPlugin` stays (keeps `move_player`);
   geometry consts stay put.

## Explicitly not doing (and why)
- **No `spawn_level!{…}` macro.** Required Components + `impl Bundle` constructors already collapse
  a spawn to `commands.spawn(static_obstacle(t, c))`. A macro adds a private DSL with no IDE
  completion / worse errors / maintenance, in a workspace with zero macros today. Use a plain
  sequence of constructor calls instead.
- **No Bevy Scenes / `DynamicScene` now.** `Collider` wraps parry2d shapes, is built fallibly, and
  is not `Reflect`/`Serialize`. Scenes also snapshot *concrete* components, but render *derives*
  meshes from `Collider` at runtime — you'd be serializing inputs anyway.
- **Deferred — generation (the real "thousands" path):** a system that scatters instances over the
  heightfield from a seed + density, calling the *same* constructors in a loop. Stores rules, not rows.
  See "Scaling placement" (Feeder 2). Not in scope now.
- **Deferred — RON authoring (when hand-laying set pieces hurts):** add a reflectable/`Deserialize`
  `ColliderSpec` mirror in `hitboxes_rapier` + a `LevelSpec` RON + loader in `level`, feeding the *same*
  constructors. A uniform RON list needs a closed `PlacementKind` enum — note the tension with
  open-ended kind growth before committing. Localized: `level` is the only caller. Not in scope now.

## Verification
- `./bin/housekeeping.sh` clean (clippy gate + fmt) after each migration step.
- `cargo run -p pathfinding`: all 4 obstacles (2 colours: static vs pushable), 4 walls, player
  render exactly as before; player collides with statics and pushes quad/pentagon; no visible
  resting overlap.
- Per-kind constructor unit tests (spawn into a minimal `App`, assert component set incl.
  require-injected `Solid`/`Static`/`MeasuredVelocity` and that pushables lack `Static`).
- `level` integration test (cashes in the lib boundary): run `spawn_level` into a minimal headless
  `App` (no `DefaultPlugins`/render), assert 4 obstacles + 4 walls + 1 player spawned and that
  pushables lack `Static`. Impossible if placement lived in `app` — this is why `level` is a crate.
