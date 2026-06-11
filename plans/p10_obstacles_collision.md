# p10 — obstacles & collision crate evaluation

Evaluation of `crates/collision`, `crates/obstacles`, and `crates/render/src/obstacle` after the
p9 SAT rewrite and p10 obstacle spawn work.

**Verdict:** solid. The pure-geometry core (`hull`/`sat`/`world`/`manifold`) is cleanly separated
from ECS, well-documented, well-tested; the type-carries-proof `ConvexHull` is the right design.
Findings below, ranked.

## Correctness / robustness

### 1. Single-pass, stale-manifold resolution (`response.rs`) — fix first
All events are detected from frame-start positions, then applied sequentially. If the player
overlaps two walls at a corner, resolving pair 1 moves it, then pair 2's depth is stale —
over/under-correction, visible as corner jitter or squeezing through. Pushable chains
(player → quad → pentagon) are worse: the 0.5/0.5 split never propagates within a frame.
Fix: iterate detect→resolve 2–4 times, or re-query depth at resolve time. Only item on the list
producing visibly wrong behavior.

### 2. Tunneling
Pure positional correction in `Update`: a fast mover or frame hitch can step over a thin wall.
`PLAYER_SPEED * dt` vs wall thickness currently has margin, but nothing enforces it.
Cheap mitigation: collision in `FixedUpdate`. Real fix: swept tests. Defer, but note in code.

### 3. Duplicate consecutive points survive hull construction (`hull.rs`)
Test even asserts len 5. Downstream: degenerate zero-length edges SAT must skip, degenerate
triangles in the render fan. Dedup in `ConvexHull::new` (`dedup_by`); the skip in `edge_normals`
becomes insurance instead of load-bearing.

## Performance (not urgent at current N, but N will grow)

### 4. `to_world` recomputed per pair (`narrow_phase.rs`)
Entity in k overlapping pairs rotates its hull k times per frame; convex case already
world-transformed once in `rebuild_spatial_hash` for the AABB. Cache `WorldShape` (and AABB)
per entity per frame.

### 5. No AABB early-out before SAT
Same-cell pairs go straight to full SAT. Min/max overlap check first rejects most pairs for four
comparisons — relevant because cells (80) are smaller than obstacles (~130 span), so cell-sharing
is common.

### 6. Statics rebuilt into the hash every frame (`spatial_hash.rs`)
Walls/static obstacles never move; only dynamics need re-insertion. Split static cells (rebuild
on change) from a per-frame dynamic layer. Also `cells.clear()` drops every `Vec` allocation —
clear vecs in place, retain keys; reuse the `seen` HashSet across frames.

## API / architecture

### 7. `render_size` lives in the collision crate (`components.rs`)
Rendering depends on logic, never the reverse — a render-named method on `Collider` is the soft
violation, and its origin-centered assumption for convex hulls is a documented trap. Rename to
local-AABB semantics (`local_extent()` returning actual min/max so off-center hulls aren't
silently wrong); render derives sprite size. Same flavor: `OBSTACLE_Z` in
`obstacles/constants.rs` is render layering in a logic crate — pragmatic since Transform carries
z, but the pattern is accumulating; consider render owning z assignment.

### 8. `CollisionEvent` flattens `Manifold` (`events.rs`)
`pub manifold: Manifold` instead of duplicated `normal`/`depth`. One contact type; manifold can
grow (contact point, feature ids) without touching the event.

### 9. `Entity` is `Ord` (`narrow_phase.rs`)
`a.index() < b.index()` dance → `if a < b { (a, b) } else { (b, a) }`. Same semantics, less to
explain.

### 10. `ConvexHull` validates but doesn't hull (`hull.rs`)
Name promises monotone chain; behavior is a validator. Rename (`ConvexPolygon`) or add
`from_points` that computes the hull from a cloud — wanted anyway when navmesh/pathfinding
starts inflating obstacle geometry.

## Consolidation

### 11. Three parallel "attach visuals" systems
Player and wall attach sprites from `Collider::render_size`; obstacles build meshes from
`collider.shape`. The obstacle path (mesh straight from collider, single source of truth) is
strictly better; walls and player could use it, deleting the sprite paths and `render_size`.

## Obstacles crate
Nothing to criticize — ~100 lines of spawn data doing its job. Hardcoded placement constants
fine for dev; data-driven placement is a later problem.

## Suggested order
1. #1 now.
2. #3, #9 — five-minute cleanups.
3. #7, #11 — next time render is touched.
4. #4–#6 — when entity count grows.
5. #2, #10 — future work.
