# p14 — Extract `hitboxes` crate (spawn vocabulary) from `collision`

## Context

World-building code (walls, obstacles, player spawn) describes objects' collision volumes and
policy by attaching components; the collision crate consumes them to detect/resolve. Today both
live in one crate, so world/player depend on the whole detection machinery just to say "this is
a solid box." Split the spawn-time vocabulary — the data that changes as the world is altered —
into its own crate `hitboxes`; `collision` becomes pure machinery that reads it.

Resulting graph (cycle-free home for future policy components like PassThrough/ZBand/WallState
sync — see FUTUREFEATURES.md filtering note):

```
hitboxes  ←  collision
    ↖         ↗
   world, player
```

Decided in conversation: name is `hitboxes`; policy markers (`Solid`, `Static`) move too, not
just shape data. Detection math (aabb, grid, SAT, manifold, WorldShape/to_world, world_aabb)
stays in collision — it's runtime machinery, not spawn data.

## What moves (collision → hitboxes)

| Item | From | To |
|---|---|---|
| `Collider` (+ `obb`/`circle`/`convex`/`render_size`), `Solid`, `Static` | `collision/src/components.rs` | `hitboxes/src/components.rs` |
| `ColliderShape` enum | `collision/src/shape.rs` | `hitboxes/src/shape.rs` |
| `ConvexHull`, `HullError` (+ tests) | `collision/src/hull.rs` | `hitboxes/src/hull.rs` |
| `HULL_COLLINEAR_EPSILON` (and any other constant only the moved code uses — verify at move time) | `collision/src/constants.rs` | `hitboxes/src/constants.rs` (per-module constants policy) |

Stays in collision: `world_aabb` (remains in `collision/src/shape.rs`, now importing
`ColliderShape` from hitboxes), `WorldShape`/`to_world`, aabb, grid, sat, manifold,
narrow/broad phase, static_index, solver, events, plugin, all pipeline constants
(`COLLISION_EPSILON`, `SOLVER_ITERATIONS`, `PENETRATION_*`, `BROAD_PHASE_MARGIN`, `CELL_SIZE`).

## Steps

1. **Create crate** `crates/hitboxes`: `Cargo.toml` (bevy workspace dep, same pattern as
   collision; smallvec only if moved code needs it — check), `src/lib.rs` with module
   declarations + re-exports only (no items in lib.rs, project rule).
2. **Move files** with plain `mv`/`cp` (never `git mv` — user handles VCS): hull.rs,
   components.rs; split `ColliderShape` out of collision's shape.rs into hitboxes/src/shape.rs.
   Move `HULL_COLLINEAR_EPSILON` into new `hitboxes/src/constants.rs`.
3. **Rewire collision**: add `hitboxes` to `collision/Cargo.toml`; update `collision/src/lib.rs`
   (drop moved modules); fix imports in shape.rs, world.rs, broad_phase.rs, static_index.rs,
   narrow_phase tests, solver tests (`use hitboxes::components::{Collider, Solid, Static}`,
   `use hitboxes::shape::ColliderShape`, etc.).
4. **Rewire consumers**:
   - `crates/world`: Cargo.toml swaps `collision` → `hitboxes`; `wall.rs:5` import path.
   - `crates/player`: Cargo.toml adds `hitboxes` (keeps `collision` — `plugin.rs` uses
     `CollisionSet`); `systems.rs` imports `Collider`, `Solid` from hitboxes.
   - `crates/app`: unchanged (only `CollisionPlugin`/`CollisionSet`).
   - Workspace root `Cargo.toml`: add member + `[workspace.dependencies]` entry.
5. **Tests**: hull tests move with hull.rs; ColliderShape/`render_size` tests (if any) move;
   collision's remaining tests keep working via new imports.

No behavior change anywhere — pure relocation; public item names/APIs identical, only paths move.

## Verification

- `cd /home/isaak/RustroverProjects/pathfinding && ./bin/housekeeping.sh` (clippy + fmt, zero
  warnings — required by project).
- `cargo test --workspace` (hull/shape tests now under hitboxes; collision suite unchanged).
- `cargo run -p pathfinding` — player collides with walls/obstacles exactly as before.

## Out of scope (recorded, not done here)

- p13 finding 1 (`Solid` staleness fix via live `Has<Solid>` read) — separate change, lands
  after this; only its import paths are affected by the move.
- Renaming collision's confusingly-named `world.rs` module (world-space shapes) — optional
  follow-up, not bundled to keep this diff pure-move.
