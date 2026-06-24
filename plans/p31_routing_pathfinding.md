# Plan: Grade-constrained pathfinding crate (`routing`)

## Context

We want to connect two world points with rail by **routing**, not hand-authoring
waypoints. The route must respect a maximum **grade** (elevation delta over horizontal
distance) — rail can't climb arbitrarily steep terrain. This same routing is wanted later
for roads (looser grade cutoff) and off-road agents, so the algorithm must be **generic
over a grade profile**, not rail-specific.

The two seams already exist:
- **Producer side:** `RailTrack::new(authored: Vec<Vec2>)` (`crates/rail/src/track.rs:32`)
  is the single seam every track producer goes through; it Catmull-Rom smooths the
  waypoints. A pathfinder just needs to hand it a `Vec<Vec2>`.
- **Elevation side:** `HeightField` (`crates/world/src/elevation/height_field.rs:14`)
  is generated procedurally at startup and queryable via `sample(Vec2) -> f32`. There is
  **no heightmap file** — the recipe lives in `assets/level.ron`, the values only in the
  in-memory `HeightField` resource. So routing queries the live field.

**Decisions locked in** (from discussion): hard grade cutoff + mild cost weighting for
steeper-but-valid edges; external `pathfinding` crate for A*; new crate named `routing`;
algorithm + API only (no in-game placement wiring this pass); elevation accessed through a
trait so `routing` stays a pure, independently testable leaf.

## Deliverable

A new `crates/routing` crate exposing:

```rust
pub fn find_path(
    field: &impl ElevationSampler,
    start: Vec2,
    goal: Vec2,
    profile: &PathProfile,
) -> Option<Vec<Vec2>>   // None when no grade-legal route exists
```

Callers (rail today, roads/agents later) build a `PathProfile`, call `find_path`, and feed
the result straight into their producer — e.g. `RailTrack::new(find_path(..)?)`. Wiring that
call into rail placement/UI is explicitly **out of scope** for this pass.

## Architecture

`routing` is a leaf crate depending only on `bevy` (for `Vec2`/`IVec2`/`Rect`) and the
external `pathfinding` crate. It knows nothing about rail or `world`.

Elevation comes in through a trait defined in `routing`:

```rust
pub trait ElevationSampler {
    fn height(&self, p: Vec2) -> f32;  // world-space elevation
    fn bounds(&self) -> Rect;          // map extent; search is clamped to this
}
```

`world` implements this trait for `HeightField` (one small file), so `world` gains a
dependency on the tiny `routing` leaf — clean direction (foundational data crate → pure
algorithm leaf; no rendering involved, no rule broken). Tests in `routing` use mock
samplers (flat / ramp / cliff) with zero `world` dependency.

### Algorithm

Grid A* over the elevation grid using `pathfinding::directed::astar::astar`:

- **Node:** `IVec2` grid coordinate. **Anchor:** node `(0,0)` is `bounds().min`; world
  pos = `bounds().min + node.as_vec2() * step`, and `node = round((p - bounds().min) / step)`.
  `step` (world units between nodes) defaults to `ELEVATION_CELL` (10.0), so when the caller
  keeps the default the search grid lines up with the height cells. (`sample` is bilinear, so
  off-default steps still work — this just avoids needless resampling within a cell.)
- **Start/goal:** snapped to the nearest grid node; final path converted back to world
  `Vec2` at node centers, with the exact `start`/`goal` substituted at the ends.
- **Successors:** 8-connected neighbours. A neighbour whose node falls **outside `bounds()`
  is dropped**, not clamped — clamping an edge node back onto its source would create a
  zero-length edge and `grade = |Δh| / 0` (NaN). For a surviving edge a→b:
  `grade = |height(b) - height(a)| / horizontal_dist`. If `grade > profile.max_grade` the
  edge is **dropped** (hard cutoff). Otherwise it's kept.
- **Cost (`u64` micro-units — `astar`'s cost type needs `Ord + Zero + Copy`, floats give
  neither `Ord` nor determinism):**
  `cost = round(horizontal_dist * (1 + grade_cost_weight * grade / max_grade) * COST_SCALE)`
  with `COST_SCALE = 1000`. `u64` (not `u32`) so a long path across a large map can't
  overflow the accumulated cost. Distance term makes shorter paths win; the grade term
  gently biases toward flatter ground among legal edges.
- **`max_grade` must be `> 0`** (debug_assert in `find_path`): the cost's `grade / max_grade`
  divides by it, and a non-positive cutoff drops every non-flat edge anyway. Caller contract,
  not a runtime branch.
- **Heuristic:** straight-line horizontal distance start→goal in the same micro-units, i.e.
  `dist * COST_SCALE`. Every edge's real cost is `≥ horizontal_dist * COST_SCALE` (the grade
  factor is `≥ 1`), and straight-line ≤ summed edge distance, so `h ≤` true remaining cost ⇒
  admissible ⇒ A* stays optimal.
- **Simplify:** Ramer–Douglas–Peucker on the raw node path to thin the ~10 m-spaced points
  before handing off, so `RailTrack`'s Catmull-Rom isn't fed a dense jagged polyline.
  Endpoints preserved. **Tolerance ≈ `step`** (not arbitrarily small): 8-connected A* can only
  move in 45° increments, so a diagonal route comes out as a single-cell *staircase*, which is
  not collinear and survives a tight RDP. A staircase's perpendicular deviation from its chord
  is `≤ step/2`, so a tolerance around `step` flattens it into the clean diagonal while still
  preserving genuine bends. `SIMPLIFY_TOLERANCE` lives in `constants.rs`.

```rust
pub struct PathProfile {
    pub max_grade: f32,         // hard cutoff |Δh|/horizontal_dist (rail strict, road looser)
    pub grade_cost_weight: f32, // mild penalty for steeper-but-legal edges
    pub step: f32,              // grid spacing for the search (default ELEVATION_CELL)
}
```

## Files

**New crate `crates/routing`** (per constants policy + "no items in mod/lib.rs"):
- `Cargo.toml` — deps: `bevy` (workspace), `pathfinding = "4"`. Note: the workspace `bevy`
  pulls in render/winit/ui features this leaf never uses, just for `Vec2`/`IVec2`/`Rect`.
  Matching the existing repo convention (every leaf takes workspace `bevy`) keeps the `Vec2`
  type identical to what `RailTrack::new` expects, so we accept the heavier dep rather than
  introduce a separate `bevy_math`/`glam` line and risk a version skew on the shared `Vec2`.
- `src/lib.rs` — `mod`/`pub use` only.
- `src/sampler.rs` — `ElevationSampler` trait.
- `src/profile.rs` — `PathProfile` struct (+ a documented rail-ish default constant).
- `src/grid.rs` — node↔world snap/convert helpers (anchored at `bounds().min`); consumes the
  neighbour-offset table from `constants.rs` (does not redefine it).
- `src/astar.rs` — `find_path`: builds successors/heuristic, runs `astar`, returns world path.
- `src/simplify.rs` — Ramer–Douglas–Peucker.
- `src/constants.rs` — `DEFAULT_STEP` (= `ELEVATION_CELL`'s value, `10.0`; copied not imported
  to keep the leaf `world`-free — comment the coupling), `SIMPLIFY_TOLERANCE`, `COST_SCALE`,
  and the 8-neighbour offset table.

**`crates/world`** (implement the trait):
- `src/elevation/sampler.rs` (new) — `impl ElevationSampler for HeightField`: `height` →
  `self.sample(p)`; `bounds` → `Rect::from_corners(origin, origin + (dims-1)*cell)`. Needs a
  small private accessor or direct field use inside the crate (fields are private but the
  impl lives in `world`, so direct access is fine).
- `src/elevation/mod.rs` — add `mod sampler;`.
- `Cargo.toml` — add `routing = { workspace = true }`.

**Workspace `Cargo.toml`:**
- Add `"crates/routing"` to `members`.
- Add `routing = { path = "crates/routing" }` and `pathfinding = "4"` to
  `[workspace.dependencies]`.

**No changes** to `rail` this pass (scope = algorithm + API only).

## Edge cases

- No legal route (e.g. a cliff walling off the goal): `astar` returns `None` → `find_path`
  returns `None`. Caller decides what to do.
- Start/goal snap to the same node, or are within one step: return the two **world** endpoints
  (`start`, `goal`) directly without searching.
- `start == goal` (or closer than `RailTrack`'s `MIN_SEGMENT_LEN`): the result is a
  zero-length 2-point path that would trip `RailTrack::new`'s `acc > 0` debug_assert. This is
  a caller error, not routing's to paper over — document that `find_path` requires
  `start != goal`; optionally return `None` for a sub-`MIN_SEGMENT_LEN` separation so the
  contract is enforced at the seam rather than blowing up downstream. Decide which in review.
- Start/goal outside `bounds()`: the endpoint node is snapped to the nearest in-bounds node
  for the search; the exact (out-of-map) `start`/`goal` are still substituted back at the
  ends of the returned path. Document that out-of-map endpoints clamp into the searched grid.
- Non-degenerate result (≥2 distinct points) satisfies `RailTrack::new`'s contract.

## Verification

- `cargo test -p routing` — unit tests with mock samplers:
  - flat field → near-straight path, start/goal preserved;
  - gentle ramp under cutoff → path crosses it; ramp over cutoff → routes around it;
  - full-width wall above cutoff with no gap → `None`;
  - simplify thins a long collinear run to its endpoints.
- `cargo test -p world` — trait impl: `height`/`bounds` agree with `sample` and dims/origin.
  `world` has `routing` but **not** `rail`, so a real-field test here can run `find_path`
  against a `HeightField` and assert a non-degenerate path — it cannot also build a
  `RailTrack`.
- `cd /home/isaak/RustroverProjects/pathfinding && ./bin/housekeeping.sh` — clippy + fmt
  clean, no warnings.
- Cross-crate sanity (optional, **out of scope this pass**): the only crate with `world` +
  `routing` + `rail` together is `app`, so the `find_path → RailTrack::new` end-to-end check
  belongs there, alongside the actual placement wiring — not in this algorithm-only pass.
  Keep `routing` and `world` tests dependency-pure.

<!-- auto-reviewed -->