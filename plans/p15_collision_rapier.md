# p15: Rapier2d collision — `collision_rapier` + `hitboxes_rapier`

## Context

The hand-rolled collision stack (grid broad phase, static index, SAT narrow phase, hull validation) works but is custom math we maintain ourselves. Decision: replace it with **raw rapier2d 0.33** (not bevy_rapier2d — no Bevy-version coupling, we keep our scheduling/writeback). Rapier's BVH broad phase + narrow phase replace all our geometry code; our **push-out solver semantics are ported** (rapier's dynamics solver is NOT used — player keeps writing `Transform` directly, solver pushes out, same feel as today).

- New crates: `crates/hitboxes_rapier` (collider data) and `crates/collision_rapier` (detection + resolution).
- Old `crates/hitboxes` + `crates/collision` are **preserved and deprecated** (doc-comment in lib.rs, stay compiling, app stops using them).
- Dependents (world, obstacles, player, render, app) switch to the new crates.
- Versions verified: rapier2d 0.33 → parry2d ^0.28, nalgebra 0.35; Bevy 0.18 ships glam 0.30. Manual glam↔nalgebra conversion helpers; no version-coupled convert features.
- After approval: copy this plan to `plans/p15_collision_rapier.md` (repo convention).

## Step 1 — Workspace `Cargo.toml`

- `members` += `crates/hitboxes_rapier`, `crates/collision_rapier` (old members stay).
- `[workspace.dependencies]` += `rapier2d = "0.33"`, `parry2d = "0.28"` (cargo unifies `^0.28` with rapier's pin; verify single copy via `cargo tree -i parry2d`), plus path entries for both new crates.

## Step 2 — `crates/hitboxes_rapier` (deps: bevy, parry2d — no rapier)

Flat lib.rs with module decls only (CLAUDE.md: nothing in mod.rs/lib.rs).

- **`shape.rs`** — ADT over parry shapes:
  ```rust
  pub enum ColliderShape { Obb(Cuboid), Circle(Ball), Convex(ConvexPolygon) }
  ```
  Methods: `to_shared_shape() -> SharedShape`, `local_extent() -> Vec2` (full span; convex via `compute_local_aabb().extents()`), `hull_points() -> Option<Vec<Vec2>>` (Convex points for render fan mesh). All `#[must_use]`.
- **`components.rs`** — drop-in API match with old crate:
  `Collider { shape }` with `obb(half_extents)`, `circle(radius)`, `convex(points) -> Result<Self, ConvexError>`, `render_size()` (delegates to `local_extent`); markers `Solid`, `Static`.
- **`convert.rs`** — the single glam↔nalgebra seam: `vec2_to_point`, `point_to_vec2`, `vec2_to_vector`, `vector_to_vec2`, `transform_to_isometry` (angle via rotated X axis `atan2` — ports old `rotation_cos_sin` trick).
- **`constants.rs`** — likely empty or minimal; `HULL_COLLINEAR_EPSILON` dies with the validator.
- **No `hull.rs` port — semantic change (flagged):** `convex()` uses parry's `ConvexPolygon::from_convex_hull(points)`, which *computes* the hull from any point cloud. Old API rejected concave/CW input with `HullError`; new API accepts and hulls it, erroring only on degenerate input (<3 effective points / zero area) via a small `ConvexError`. Parry may also merge duplicate/collinear vertices, so `hull_points()` can return fewer points than authored. Rationale: per decision, port our code only where rapier has no solution — it has one here.

## Step 3 — `crates/collision_rapier` (deps: bevy, rapier2d, hitboxes_rapier)

Use `rapier2d::parry` re-export internally so pipeline and queries share one parry.

- **`constants.rs`** — ported with their tuning doc comments: `COLLISION_EPSILON = 1e-4`, `SOLVER_ITERATIONS = 12`, `PENETRATION_SLOP = 0.5`, `PENETRATION_PERCENT = 1.0`, `CONTACT_PREDICTION_DISTANCE = 8.0` (successor of `BROAD_PHASE_MARGIN`, same rationale). `CELL_SIZE` dies with the grid.
- **`events.rs`** — `CollisionEvent { a, b, normal, depth }` copied verbatim from old crate (a→b normal, pre-resolution depth, FixedUpdate-reader warning). Identical contract.
- **`physics.rs`** — `PhysicsWorld` Resource: `RigidBodySet`, `ColliderSet`, `IslandManager`, `BroadPhaseBvh`, `NarrowPhase`, `CollisionPipeline` + `Default`.
- **`mapping.rs`** — `ColliderMap` Resource: `HashMap<Entity, BodyBinding>` where `BodyBinding::Static { collider }` | `Kinematic { body, collider }`; `entity_to_user_data`/`entity_from_user_data` (Entity bits in `collider.user_data: u128`).
- **Body topology (REVISED during implementation):** ALL colliders are parentless (rapier-fixed) — no rigid bodies at all. Dynamics carry `ActiveCollisionTypes::FIXED_FIXED` and get their collider pose rewritten every tick. The planned kinematic-body topology panics in rapier 0.33: `CollisionPipeline::step` passes `None` for the island manager when applying body changes, so kinematic bodies never join an island, but `NarrowPhase` asserts island membership when a contact starts between two non-fixed bodies (`island_manager/manager.rs:119`). Parentless colliders have no body handles, so the island machinery is never touched. Pair activity is the OR of both colliders' flags ⇒ dynamic↔static and dynamic↔dynamic fire; static-static structurally never does.
- **`sync.rs`** — `sync_physics_world`, skip-proof for FixedUpdate frame skipping (no `RemovedComponents`, no message bookkeeping):
  1. Removals: map entries whose entity no longer matches `Query<(), With<Collider>>` → remove handles + binding.
  2. Insertions: any collider entity not in map → create binding.
  3. `Changed<Collider>` → `set_shape`; `Static` marker flipped vs binding variant → recreate binding.
  4. Positions: statics on `Changed<Transform>` only; kinematics unconditionally each tick via `body.set_position(iso, false)` (verify attached-collider pose propagates inside `step`; fallback: set collider position directly).
- **`step.rs`** — `step_collision_pipeline`: `pipeline.step(CONTACT_PREDICTION_DISTANCE, …, &(), &())`. This one call replaces grid + static index + SAT + AABB.
- **`solver.rs`** — ported push-out solver, same semantics as `crates/collision/src/solver.rs`:
  - Data: `SolverBody { entity, iso, shape: SharedShape, offset: Vector<f32> }`, `StaticAnchor`, `PairBodies::{DynamicDynamic, DynamicStatic}`, `CandidatePair { bodies, both_solid }`, `pair_weights` (0/0, 0/1, 0.5/0.5) ported verbatim.
  - Contact test for ALL passes: `parry2d::query::contact(&iso_shifted_a, …, 0.0)` with accumulated offsets added to isometry translations (replaces `WorldShape::shift`). One normal convention everywhere (sidesteps rapier manifold-direction ambiguity); penetrating when `dist < -COLLISION_EPSILON`, depth `-dist`, normal `normal1` (a→b — verified by a dedicated test written FIRST; negate in one place if test disagrees).
  - Pass 0 tests all candidate pairs and records `InitialContact` for every touching pair (solid or not) for events; passes 1..12 retest only both-solid pairs; correction `(depth - slop).max(0.0) * percent` split by weights; early-out on a no-op pass.
  - `resolve_collisions` system: gather candidates from `narrow_phase.contact_pairs()` (graph edges = old margin-inflated list since prediction = 8.0), map handles→entities via `user_data`, dynamic side ordered as `a` (preserves a→b convention), `both_solid` from `Has<Solid>` queries; run solver; emit `CollisionEvent`s; single Transform writeback for offsets with `norm > COLLISION_EPSILON`. Rapier poses go stale post-writeback within the tick — documented on `PhysicsWorld`; next tick's unconditional kinematic sync heals it.
- **`plugin.rs`** — `CollisionSet` (same name) + `CollisionPlugin`: init resources, `add_message::<CollisionEvent>`, FixedUpdate `(sync_physics_world, step_collision_pipeline, resolve_collisions).chain().in_set(CollisionSet)`. Port the fixed-tick/tunneling doc comment.

## Step 4 — Dependent migration

| File | Change |
|---|---|
| `crates/world/{Cargo.toml, src/wall.rs}` | dep + import swap to `hitboxes_rapier` — body unchanged |
| `crates/obstacles/{Cargo.toml, src/spawn.rs}` | same; `convex(...).expect(...)` unchanged |
| `crates/player/Cargo.toml` | `collision`→`collision_rapier`, `hitboxes`→`hitboxes_rapier` |
| `crates/player/src/{plugin.rs, systems.rs}` | import swaps; `move_player.before(CollisionSet)` unchanged |
| `crates/render/Cargo.toml`, `src/{wall,player}/plugin.rs` | dep/import swap; `render_size()` preserved |
| `crates/render/src/obstacle/plugin.rs` | match arms → tuple variants: `Circle(ball)` → `Circle::new(ball.radius)`, `Obb(c)` → `Rectangle::new(c.half_extents.x*2.0, c.half_extents.y*2.0)`, `Convex(_)` → `convex_mesh(&shape.hull_points()…)`; `mesh.rs` unchanged |
| `crates/app/{Cargo.toml, src/main.rs}` | `collision`→`collision_rapier`; plugin list + `MotionSet.after(CollisionSet)` otherwise unchanged |

**Deprecation:** prepend to old `hitboxes`/`collision` lib.rs: `//! **Deprecated:** superseded by hitboxes_rapier/collision_rapier. Kept for reference; do not add new dependents.` Old crates stay members and keep compiling (old collision still deps old hitboxes).

## Step 5 — Tests (all headless minimal `App`, old crates' pattern)

- **Convention gate (write FIRST):** `parry_contact_normal_points_a_to_b` — ball/ball, cuboid/cuboid, poly/circle in both argument orders; settles the `normal1` direction assumption before the solver is built.
- **hitboxes_rapier:** convex from concave cloud → hulled (new semantics); degenerate → error; `render_size` all variants incl. off-center convex; `to_shared_shape` smoke; `transform_to_isometry` rotation+translation (port of `world_poly_rotates_then_translates`).
- **solver unit (port ~1:1 from old solver.rs):** dynamic_pushed_out_of_static_opposite_normal, corner_two_statics_resolves_both, chain_push_propagates_to_static, separated_pair_pass_zero_only, overlap_within_slop_not_corrected, symmetric_dynamic_pair_splits_evenly, non_solid_overlap_reported_not_resolved.
- **integration (3-system chain):** sync lifecycle (spawn/despawn → handles + map consistent); `Changed<Collider>`/static `Changed<Transform>` propagate; static-static overlap → zero events/movement; end-to-end event with a→b normal + push-out past slop; rotated-OBB and off-center-convex push-out; pair-in-graph-within-prediction but NOT touching → no event; `ActiveCollisionTypes` behavior asserted empirically.
- Old tests that die with their machinery: grid/static_index/broad_phase internals, SAT, `world_aabb`, `WorldShape::translate`, hull validation.

## Risks

1. `normal1` direction — gated by the convention test, single negation point if wrong.
2. Prediction (8.0) widens the pair graph only; the touching gate is `dist < -COLLISION_EPSILON` at prediction 0.0 — never mix in slop/prediction or events fire for non-touching pairs.
3. Kinematic `set_position` → collider pose propagation inside `CollisionPipeline::step`: verify; fallback sets collider positions directly.
4. parry version skew between crates: verify one `parry2d` in `cargo tree`; fallback: hitboxes_rapier uses `rapier2d::parry` re-export.
5. Concave-input semantics change (hulled, not rejected) — intentional, see Step 2.

## Verification

1. `cargo test --workspace` — ported solver tests + new integration tests green.
2. `./bin/housekeeping.sh` — clean clippy/fmt, no warnings.
3. `cargo run -p pathfinding` — walk player into walls/obstacles: pushed out smoothly, slides along walls, pushable quad/pentagon obstacles still shove, static circle/triangle immovable, visuals match colliders (sprite sizes, convex meshes).
