# Plan: SAT collision — OBB / convex / circle with manifold events

## Context

The collision crate (`crates/collision/src`) is AABB-only: `Collider { half_extents: Vec2 }`,
an axis-aligned overlap test, and a response that recomputes the MTV. Nothing in the project
uses `Transform.rotation` yet, so the system can't handle rotated or non-rectangular colliders —
which we'll need as obstacles/terrain stop being axis-aligned.

Goal: replace the narrow phase with a unified shape system supporting **oriented boxes (OBB),
convex polygons, and circles**, drop the AABB-specific fast path, and have collision detection
emit a **contact manifold (normal + penetration depth)** so response is shape-agnostic.

Decisions (from the user): all three shapes; one unified narrow phase (no AABB special case —
boxes are just zero-or-nonzero-rotation OBBs); `CollisionEvent` carries the manifold.

## Sequencing: collision engine first, shapes are data after

We have one shape today (axis-aligned square, no rotation). The question is whether to add
non-box/round/tilted shapes to gameplay first, or build SAT first. **SAT first** — it's the
prerequisite, not the dependent:

- The moment any entity is rotated or non-rectangular, AABB collision is *wrong*. You can't
  gameplay-test a tilted or round shape without collision already handling it. So the engine
  must land before (or with) the first non-axis-aligned shape, never after.
- The SAT engine is fully verifiable in isolation (pure `sat.rs` unit tests, no ECS, no gameplay).
- The migration is **behavior-preserving**: the existing square becomes a zero-rotation
  `Collider::obb`, so the running game looks and plays identically. The three shapes exist in the
  type system but gameplay keeps using the square until we choose to spawn others.

So this plan ships the *engine* (SAT + manifold + `ColliderShape` enum, square migrated to OBB).
Introducing actual rotation / circles / convex hulls into entities is then **pure data** — drop a
different `ColliderShape` on a spawn, no further collision work. Those are separate follow-up steps,
out of scope here.

## Normal-sign convention (single source of truth)

Every narrow-phase result has a **unit `normal` pointing from `a` toward `b`** and `depth >= 0`.
Response defines `push = normal * depth` (a→b) and applies `translation += push * factor` with the
factor table re-signed for this convention (see Response below). This sign mapping is the most
error-prone line — pin it with a test.

## Files

### New
- `manifold.rs` — `#[derive(Clone, Copy, Debug)] struct Manifold { normal: Vec2, depth: f32 }`. Pure data.
- `sat.rs` — pure functions on world-space geometry (no ECS), each `-> Option<Manifold>`:
  - `poly_poly(a: &[Vec2], b: &[Vec2])` — SAT over both polygons' edge normals; min-overlap axis is the MTV; orient a→b by projecting `(centroid_b - centroid_a)` onto the min axis and flipping the axis so the projection is positive. **Degenerate case:** when `|projection| <= COLLISION_EPSILON` (symmetric/nested overlap — centroid delta ⊥ min axis), keep the raw SAT-axis direction instead of trusting `signum` of noise. `f32::signum(0.0) == 1.0` so the normal never zeroes, but without the guard the sign can flip frame-to-frame and jitter. Pin with a symmetric-overlap test. **Winding is irrelevant to correctness**: SAT projects onto each axis symmetrically and the final normal is re-oriented by the centroid sign — the CCW convention below is a sanity invariant only, not load-bearing for the normal math.
  - `poly_circle(poly: &[Vec2], c: Vec2, r: f32)` — SAT over polygon edge normals **plus** the closest-vertex→center axis (handles circle-vs-corner). The vertex axis is skipped when (a) it is zero-length (center on a vertex — length guard) or (b) the center is interior to the polygon (all edge projections overlap → a face normal is the min axis and wins anyway).
  - `circle_circle(ca, ra, cb, rb)` — center-distance test; coincident-center fallback to `Vec2::X`.
- `shape.rs` — `ColliderShape` enum + bounds helper:
  ```rust
  pub enum ColliderShape {
      Obb { half_extents: Vec2 },
      Convex { points: Vec<Vec2> }, // local-space, CCW, convex
      Circle { radius: f32 },
  }
  ```
  - `world_aabb(shape, transform) -> (Vec2, Vec2)` (min, max) for the broad phase. Circle = ±radius; Obb = `(|cos|*hx+|sin|*hy, |sin|*hx+|cos|*hy)`; Convex = min/max of rotated points (handles off-center hulls). Rotation via the shared `world.rs` helper.
- `world.rs` — `to_world(shape, transform) -> WorldShape` where
  ```rust
  pub enum WorldShape {
      Poly(SmallVec<[Vec2; 8]>), // inline up to 8 verts: boxes (4) and typical hulls stay on the stack
      Circle(Vec2, f32),
  }
  ```
  plus `world_poly(local: &[Vec2], transform) -> SmallVec<[Vec2; 8]>`. Apply rotation as `(cos,sin)` from `(rot * Vec3::X).truncate()` (avoids euler edge cases) then `+ translation.truncate()`. Obb generates its 4 CCW corners first, then same path. **`SmallVec<[Vec2; 8]>` keeps boxes and small hulls alloc-free in the hot path; only hulls with >8 verts spill to the heap.** The `(cos,sin)` extraction + `local → world` vertex transform lives in **one shared helper** here, reused by `shape::world_aabb` (no second copy of the rotation trick).

### Modified
- `components.rs` — `Collider { shape: ColliderShape }`; `#[must_use]` constructors `obb(half_extents)`, `circle(radius)`, `convex(points)` (`debug_assert!` len>=3 + convex check — **all consecutive edge cross-products share one sign, treating zero (collinear verts) as compatible with either** — documented invariant; a strict same-sign test would reject legal hulls that include collinear points); `render_size(&self) -> Vec2` returning the **full span** for sprite sizing (`Obb`→`half*2`, `Circle`→`splat(r*2)`, `Convex`→local-AABB `max - min`). **Caveat:** sprites are centered on the transform, so `render_size` only places an off-center convex hull correctly if its local AABB is origin-centered. Document that convex render assumes origin-centered hulls; offsetting the sprite to the AABB center is out of scope here. Likewise `Circle` renders as a **square** sprite (`splat(r*2)`) — collider rendering is approximate; true round/centered rendering is out of scope.
- `events.rs` — `CollisionEvent { a, b, normal: Vec2, depth: f32 }`.
- `narrow_phase.rs` — delete `aabb_overlaps`; add `test_pair(a_shape, a_tx, b_shape, b_tx) -> Option<Manifold>` dispatch (Poly/Poly, Poly/Circle, Circle/Poly→swap+negate normal, Circle/Circle). `detect_collisions` keeps the dedup HashSet + static-static skip, emits the manifold. Treat min-overlap `<= COLLISION_EPSILON` as `None` (no jitter on grazing contact). **Invariant: the manifold normal is computed for the same `a,b` stored in the event (loop order), NOT the index-ordered dedup key** — the key exists only to dedup the HashSet; storing the ordered pair instead of loop `a,b` would silently invert every normal in the game. Guard with the swap-negate symmetry test.
- `spatial_hash.rs` — `rebuild_spatial_hash` uses `world_aabb` → derive `center = (min+max)/2`, `half = (max-min)/2` → existing `cells_for_aabb` (signature unchanged). **Feed the derived AABB center, not `transform.translation`** — for off-center convex hulls they differ.
- `response.rs` — delete `aabb_mtv`; read `normal,depth`; `push = normal*depth`; apply `a += push*a_factor`, `b += push*b_factor` with re-signed table `(true,false)=>(0,1)`, `(false,true)=>(-1,0)`, `(false,false)=>(-0.5,0.5)`, `(true,true)=>continue`. **Verify with the response test** (player hits right wall → pushed left).
- `constants.rs` — add `pub const COLLISION_EPSILON: f32 = 1e-4;` (alongside existing `CELL_SIZE`). It serves double duty — normalize/length guard *and* the touching→`None` penetration gate. Both want `1e-4` today; split into two named constants only if the scales ever need to diverge (a length guard and a penetration threshold aren't the same dimension).
- `lib.rs` — add `pub mod manifold; pub mod sat; pub mod shape; pub mod world;`.
- `Cargo.toml` — add `smallvec` dependency (declare in workspace `[workspace.dependencies]` and reference as `smallvec = { workspace = true }`, matching the existing `bevy` pattern). Backs `WorldShape::Poly`.
- `plugin.rs` — no change (system chain identical).

### Call-site migration (mechanical)
- `crates/player/src/systems.rs:15` — `Collider { half_extents: ... }` → `Collider::obb(Vec2::splat(PLAYER_SIZE/2.0))`.
- `crates/world/src/wall.rs:23` — `Collider { half_extents }` → `Collider::obb(half_extents)`.
- `crates/render/src/player/plugin.rs:21` & `crates/render/src/wall/plugin.rs:21` — `collider.half_extents * 2.0` → `collider.render_size()` (keeps render reading from collision, one-directional).

## Edge cases
- Coincident centers → fallback axis `Vec2::X`, depth = sum of radii/extents (guard with `COLLISION_EPSILON`).
- Touching/zero-depth → `None`. Deep penetration → SAT min-axis is correct by construction (no CCD; tunneling out of scope).
- Convex winding does **not** affect SAT correctness (symmetric projection + centroid re-orientation). `debug_assert` convexity (consistent cross-product sign) as an invariant; Obb corners emitted CCW for consistency, not because the math needs it.
- Normalize normals with a length guard; all overlap comparisons use `> COLLISION_EPSILON`.

## Module rules compliance
No code in mod.rs (none exist — flat files + `lib.rs` decls only). Epsilon lives in the crate's
`constants.rs` (collision-wide tolerance). Constructors/`render_size` sit with the `Collider` type.

## Implementation order
0. `Cargo.toml` — add `smallvec` (workspace + crate)
1. `manifold.rs` + `constants.rs` epsilon
2. `sat.rs` + unit tests (green before ECS)
3. `shape.rs` + `world.rs` + bounds/world-vertex tests
4. `components.rs` (enum, constructors, render_size)
5. `events.rs` (manifold fields)
6. `narrow_phase.rs` (test_pair, drop aabb_overlaps, emit manifold)
7. `spatial_hash.rs` (world_aabb)
8. `response.rs` (drop aabb_mtv, apply manifold, **verify sign**)
9. `lib.rs` decls
10. Call sites (player, wall, 2× render)
11. `./bin/housekeeping.sh`, fix all warnings

## Verification
- `cargo test -p collision` — per-module `#[cfg(test)]` tests (convention: `chunk_coord.rs`):
  - sat: box/box overlap normal+depth, rotated overlap, edge-touch→None, circle-face vs circle-corner normals, circle/circle depth + coincident fallback, **symmetric/nested overlap → stable non-jittering normal (degenerate-orientation guard)**.
  - narrow_phase: **`test_pair(circle, poly).normal ≈ -test_pair(poly, circle).normal`, equal depth (swap-negate + event-order sign symmetry)**.
  - shape: `world_aabb` for axis-aligned obb, 45° square (`splat(hx*√2)`), circle (rotation-invariant), off-center convex.
  - response: manifold normal +X depth d with (a dynamic, b static) ⇒ a moves −d on X.
- `./bin/housekeeping.sh` (clippy + fmt) clean.
- `cargo run -p pathfinding` — player still collides with the 4 walls (no tunneling/jitter), sprites still sized correctly via `render_size()`.

## Risk
Dropping AABB means the common box-box path runs full SAT each frame. Negligible at current
entity counts, and the spatial hash still filters candidates. Mitigation: stack-backed vertex
buffers (no heap alloc for boxes). Defer any near-zero-rotation AABB short-circuit unless
profiling demands it (it's the special case the user chose to drop).

Secondary: `test_pair` calls `to_world` per candidate pair, so an entity in K broad-phase pairs
is transformed K times/frame (plus once in `rebuild_spatial_hash`). Acceptable at current counts;
the future win is a per-entity per-frame `WorldShape` cache. Deferred — don't build it now.
