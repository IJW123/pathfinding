# p13 — Collision rewrite review findings (post-p12)

## Context

Multi-angle review (7 finder passes, adversarial verification per candidate) of the collision
rewrite committed after p12: new `broad_phase.rs`, `solver.rs`, `static_index.rs`, `grid.rs`,
`aabb.rs`; deleted `response.rs`, `spatial_hash.rs`. Core solver and broad-phase math survived
verification cleanly. Six findings ranked below; refuted candidates recorded at the bottom so
they don't get re-litigated. Fix for finding 1 is planned here; 3–6 are batched cleanup; the
geometry-crate extraction is a separate plan (p14).

## Findings

### 1. Static index goes stale when `Solid` is toggled — fix now

`static_index.rs:29-44`. The rebuild trigger is
`!changed.is_empty() || all.iter().count() != index.entries.len()` where `ChangedStatics`
filters `Or<(Changed<Transform>, Changed<Collider>)>`. The index caches `solid: bool`
(snapshot of `Has<Solid>`, `static_index.rs:42`), and broad phase trusts it
(`broad_phase.rs:102`). Neither detection path sees a `Solid` add/remove: marker toggling
touches no watched component, and query membership (count) is unchanged.

Failure: wall spawned solid → `entity.remove::<Solid>()` (breakable wall, door) → dirty stays
false → `entry.solid` stale → solver keeps pushing bodies out of a passable wall. Inverse for
add. Latent today (`Solid` only ever set in spawn bundles) but silent the moment a toggler
appears — and pass-through gameplay is planned.

Root cause framing: the cache holds a field its invalidation watch-list doesn't cover.
Two ways to realign:

- **Option B — solid-count in the dirty check.** Cache solid count next to entry count;
  `dirty |= solid_count != cached`. Skip-proof like the entity count. Edge case: same-tick
  swap (remove `Solid` from A, add to B) keeps the count equal and is missed. Cheap but the
  hole is real.
- **Option C — stop caching `solid` (chosen).** Drop `solid` from `StaticEntry`. In
  `collect_collision_pairs`, add `solids: Query<Has<Solid>>` and read it live per candidate
  static pair (`solids.get(entry.entity).unwrap_or(false)`). Index then caches only geometry —
  exactly what `Changed<Transform>/Changed<Collider>` correctly invalidates. Staleness becomes
  unrepresentable instead of better-detected. Cost: one entity lookup per dynamic-static
  candidate pair per tick (a handful at current scale); geometry stays fully cached so the
  "no ECS touch" property is only relaxed for the policy bit, which is the part that must be
  fresh. Note `Solid` toggling never changes geometry — AABBs/shapes need no rebuild for it,
  so a full index rebuild (Option B) is also doing too much work.

Steps for C:
1. Remove `solid` field from `StaticEntry` (`static_index.rs:16`), drop `Has<Solid>` from the
   `all` query, update doc comment (index = geometry cache only).
2. `collect_collision_pairs`: add `solids: Query<Has<Solid>>`; compute
   `entry_solid` live at pair creation (`broad_phase.rs:102`).
3. Tests: update `populated_on_first_run` (asserts `entries[0].solid`); add a test toggling
   `Solid` on a static between updates and asserting `both_solid` flips without a rebuild.
4. Future `WallState -> Solid` sync system (see FUTUREFEATURES.md filtering note) composes
   unchanged: it just adds/removes the marker.

### 2. `CollisionEvent` written, never read — deliberate, keep

`solver.rs:151-158` emits for every pass-0 contact each tick; zero `MessageReader` in the
workspace. p12 records this as a deliberate keep (future sensors), and the FixedUpdate
consumer-doc landed in `events.rs`. No action. Semantics note for the first real consumer:
emission is now all-overlaps (incl. non-solid) in FixedUpdate — those choices are still
unvalidated by any consumer.

### 3. `aabb.rs` duplicates `bevy_math::Aabb2d` — optional

`overlaps`/`inflated`/`translated` map 1:1 to `intersects`/`grow`/`translated_by`
(`BoundingVolume` trait, bevy_math 0.18, zero extra deps). ~45 lines + tests of re-derived
API. Counterweight: 3-method hand-rolled struct is trivially auditable; `grow` takes `Vec2`
not scalar. Weakest finding — decide at geometry-crate extraction time (p14): if `Aabb` moves
to the new crate anyway, keeping the custom type is more defensible (crate stays
bevy_math-version-independent at its API).

### 4. Redundant inflation in dyn-dyn pair loop — fold into next touch

`broad_phase.rs:122-124` recomputes `body_a.aabb.inflated(BROAD_PHASE_MARGIN)` per candidate
pair; the same value was computed once per body at line 83 and discarded. O(pairs) redundant
work (4 float adds each — unmeasurable at current scale; the finding is that the value is in
hand and thrown away). Fix: store inflated AABB per body in a scratch Vec (or on
`DynamicBody`) and reuse. Safe: broad phase runs before any `shift()` mutation and is the only
inflated-AABB consumer.

### 5. `split_at_mut` relies on cross-module `a < b` invariant — one-liner

`solver.rs:64-66`: `split_at_mut(b)` + `left[a]`/`right[0]` requires `a < b`. Upheld by
broad phase normalization (`broad_phase.rs:115`) and mentioned in the `PairBodies` doc
(`broad_phase.rs:31`), but unasserted at the consumption site; violation panics with a bare
index-out-of-bounds pointing nowhere near the cause. Fix:
`debug_assert!(a < b, "broad phase must emit dynamic pairs with a < b");` at the top of the
`DynamicDynamic` arm. Zero release cost.

### 6. Unchecked `statics.entries[anchor]` — generation counter

`solver.rs:83,117` index the static entries with anchors captured by broad phase. Valid only
because `maintain_static_index → collect_collision_pairs → resolve_collisions` are `.chain()`ed
in one tick and `CollisionPairs` is cleared each broad phase (`broad_phase.rs:62`). Break paths:
unchaining for parallelism, persisting pairs across ticks, a mid-pipeline system dirtying a
static. Worst case is not the panic — a same-length rebuild (one despawn + one spawn) reorders
entries and the solver corrects against the wrong static's geometry, silently. Fix: `generation:
u32` on `StaticColliderIndex` bumped per rebuild, snapshot in `CollisionPairs`, `debug_assert_eq!`
in `resolve_collisions`. Converts a scheduling assumption into a checked invariant; free in
release.

## Refuted during verification (for the record)

- **Broad-phase single-sided inflation misses pairs**: algebraically impossible — gap < margin
  forces `a.max + margin >= b.min`, so the asymmetric test always passes. Covered by
  `near_contact_within_margin_enters_list`.
- **Per-pass `both_solid` re-filter wasteful**: ~120 flag checks/tick at current scale with
  early-exit; negligible.
- **Per-tick allocations**: Vecs/maps are cleared-not-dropped (caps retained) — already the
  right idiom.

## Execution order

1. Finding 1 (Option C) — correctness, do first.
2. Findings 4 + 5 + 6 — one small cleanup pass, no behavior change.
3. Finding 3 — defer to p14 (geometry crate) decision.
