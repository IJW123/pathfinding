# p12 — Collision solver cleanup: AABB early-out, grid cell leak, dead code

## Context

Post-implementation review of p11 (`plans/p11_collision_solver_fixedupdate.md`) found two perf
problems and a handful of dead/stale items. All validated in conversation; this plan executes
exactly those, nothing else.

1. **Solver re-runs full SAT without AABB gate.** `solver.rs::pair_manifold` SAT-tests every
   solid candidate pair on every Gauss-Seidel pass — including margin-only near-pairs up to 8u
   apart that can never touch. The cached AABBs are maintained incrementally by `shift()` and
   stay exact, so a 2-comparison overlap check skips most of that. The p11 plan promised an
   "AABB early-out"; it only landed in the broad phase.
2. **Dynamic grid leaks empty cells.** `broad_phase.rs::collect_collision_pairs` clears the
   `Local<HashMap>` cell `Vec`s but never removes keys. Every 80u cell a dynamic ever touched
   persists as an empty entry; memory grows with explored area and the dyn-dyn pair loop
   iterates the empties every tick.
3. **Dead/stale code** (invisible to clippy — `pub` items in a lib crate):
   - `narrow_phase::test_pair` — only caller was the deleted `detect_collisions`; test-only now.
   - `Aabb::center()` / `Aabb::half_extents()` — no production callers anywhere in workspace.
   - `events.rs` doc references the deleted `narrow_phase::detect_collisions`, and the p11 risk
     item "document that future sensors must read events in FixedUpdate" never landed.

Keep (deliberate, per conversation): `SolveOutcome`/`iterations_run` (test observability),
`CollisionEvent` emission with zero readers (future sensors), `pair_weights(true, true)` dead
arm (total function, mass slots in later).

On approval, save this plan as `plans/p12_collision_cleanup.md` (project convention).

## Steps

### 1. AABB early-out — `crates/collision/src/solver.rs`
In `pair_manifold` (line 74), check cached-AABB overlap before dispatching to `test_world_pair`;
return `None` on miss. Both arms:
- `DynamicStatic`: `bodies[body].aabb.overlaps(&statics.entries[anchor].aabb)`
- `DynamicDynamic`: `bodies[a].aabb.overlaps(&bodies[b].aabb)`
Sound because AABBs are conservative: no AABB overlap ⇒ no shape contact. Behavior identical,
existing solver tests cover it (`separated_pair_pass_zero_only` now exits via the cheap path).
Extend `pair_manifold`'s doc comment with the early-out.

### 2. Grid cell retention — `crates/collision/src/broad_phase.rs`
Replace the clear loop (lines 63–65) with a single retain-and-clear pass:
```rust
dynamic_grid.retain(|_, cell| {
    let keep = !cell.is_empty();
    cell.clear();
    keep
});
```
Cells occupied last tick keep their `Vec` allocation (hot path unchanged); cells empty for a
full tick are dropped. Memory now bounded by current occupancy, not explored area. Update the
system doc comment ("allocation retained" → retained for occupied cells).

### 3. Dead code — `crates/collision/src/narrow_phase.rs`, `aabb.rs` + test fallout
- Delete `test_pair` and its now-unused `ColliderShape`/`to_world` imports. Rewrite its test
  `test_pair_circle_poly_swap_negates_normal` against `test_world_pair` directly (lower both
  shapes with `to_world`, assert swap negates normal, equal depth — same coverage).
- Delete `Aabb::center` and `Aabb::half_extents`. Test fallout:
  - `aabb.rs::translated_moves_both_corners` — assert `min`/`max` directly.
  - `static_index.rs::transform_change_rebuilds` (line 133) — assert on `aabb.min.x` (box at
    x=100, half 10 ⇒ `min.x ≈ 90.0`).

### 4. Event docs — `crates/collision/src/events.rs`
Fix the stale `narrow_phase::detect_collisions` reference (normals now come from the solver's
pass-0 manifolds, a→b convention) and add the consumer caveat: collision runs in `FixedUpdate`;
readers must too — `Update`-schedule readers can observe the same buffer twice on 0-tick frames.

### 5. Housekeeping
`./bin/housekeeping.sh` clean; `cargo test -p collision` (44 tests, same count after the two
test rewrites).

## Verification
- All collision tests green, clippy/fmt clean.
- Behavior-neutral change set: no gameplay difference expected. `cargo run -p pathfinding`
  smoke check — push quad into pentagon/wall corner, same settling as before.

## Files
**Modified:** `crates/collision/src/{solver,broad_phase,narrow_phase,aabb,static_index,events}.rs`
