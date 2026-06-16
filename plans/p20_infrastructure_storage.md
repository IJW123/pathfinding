# Plan: `infrastructure` crate — Storage building

## Context

We want a new gameplay domain — **infrastructure** (buildings) — separate from `obstacle`
(inert level geometry). Its first member is **Storage**: a building rendered as a square that
holds four resources (grain, coal, lumber, iron ore). Unlike obstacles, storage carries *mutable
gameplay data* and must expose a way to add/remove resources at runtime so future systems
(production, transport, the player) can move goods in and out.

Architecturally it mirrors `obstacle` exactly: world-logic crate owns components/bundles/collision
markers; the `render` crate reads its components to draw it; `level` spawns it; `app` wires the
plugin. Rendering depends on infrastructure, never the reverse.

Decisions locked with the user: **message-based mutation API**, **unlimited capacity (clamp at 0)**,
**`Resource` enum + amount counts (ADT)**.

## New crate: `crates/infrastructure`

Standalone world-logic crate. Depends only on `bevy` + `hitboxes_rapier` (same as `obstacle`);
does **not** depend on `obstacle`. `lib.rs` holds only `pub mod` declarations (mirrors
`obstacle/src/lib.rs`).

```
crates/infrastructure/
  Cargo.toml            # bevy, hitboxes_rapier (workspace deps)
  src/
    lib.rs              # pub mod resource/components/message/systems/bundle/constants/plugin
    resource.rs         # Resource + ResourceChange ADTs
    components.rs        # Storage marker, Inventory (+ impl)
    message.rs          # ResourceTransfer message
    systems.rs          # apply_resource_transfers
    bundle.rs           # storage_building()
    constants.rs        # STORAGE_Z, DEFAULT_* amounts
    plugin.rs           # InfrastructurePlugin
```

### `resource.rs`
```rust
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Resource { Grain, Coal, Lumber, IronOre }

/// Signed intent over a resource. Withdraw clamps at 0 (unlimited storage, no negatives).
#[derive(Clone, Copy, Debug)]
pub enum ResourceChange { Deposit(u32), Withdraw(u32) }
```

### `components.rs`
- `Storage` — marker, `#[derive(Component)]` with `#[require(Solid)]` (so collision pipeline picks
  it up automatically, exactly like `Obstacle`). `Solid` from `hitboxes_rapier::components`.
- `Inventory` — `#[derive(Component, Default)]` holding `grain/coal/lumber/iron_ore: u32`.
  Methods: `amount(Resource) -> u32`, `add(Resource, u32)`, `remove(Resource, u32) -> u32`
  (returns amount actually removed after clamping), and `apply(Resource, ResourceChange)`.
  All mutation routes through these so the message system and any future caller share one code path.

### `message.rs` (Bevy 0.18 `Message`, per project memory — not `Event`)
```rust
#[derive(Message)]
pub struct ResourceTransfer { pub target: Entity, pub resource: Resource, pub change: ResourceChange }
```

### `systems.rs`
`apply_resource_transfers(mut reader: MessageReader<ResourceTransfer>, mut q: Query<&mut Inventory>)`
— for each message, `q.get_mut(target)` then `inventory.apply(resource, change)`. One reader, one
system, in `Update`.

### `bundle.rs`
```rust
pub fn storage_building(transform: Transform, half_extents: Vec2, inventory: Inventory) -> impl Bundle {
    (transform, Storage, Collider::obb(half_extents), inventory, Static)
}
```
`Static` because storage is immovable (same per-instance choice `static_obstacle` makes). `Solid`
comes from `Storage`'s `#[require]`. Square ⇒ `Collider::obb`.

### `constants.rs`
`STORAGE_Z` (z-layer — a kind property, like `OBSTACLE_Z`); optional `DEFAULT_*` starting amounts.
Per project constants policy: per-module `constants.rs`, no inline consts in logic.

### `plugin.rs`
`InfrastructurePlugin`: `app.add_message::<ResourceTransfer>().add_systems(Update, apply_resource_transfers);`

## Rendering: `crates/render/src/infrastructure/`

New render submodule mirroring `render/src/obstacle/`. **Reuses** `crate::obstacle::mesh::shape_mesh`
(already `pub`) to build the square mesh from the collider — no new mesh code.

```
render/src/infrastructure/
  mod.rs        # pub mod constants; pub mod plugin;
  constants.rs  # STORAGE_COLOR (distinct from obstacle colors)
  plugin.rs     # StorageRenderPlugin
```
`attach_storage_mesh(... query: Query<(Entity, &Collider), Added<Storage>>)` — inserts
`Mesh2d(shape_mesh(&collider.shape))` + `MeshMaterial2d(STORAGE_COLOR)`. One-shot at spawn, same
pattern as `attach_obstacle_mesh`.

Wiring:
- `render/src/lib.rs`: add `pub mod infrastructure;`
- `render/src/plugin.rs`: add `StorageRenderPlugin` to the `add_plugins` tuple.
- `render/Cargo.toml`: add `infrastructure = { workspace = true }`.

## Level + app wiring

- `Cargo.toml` (workspace): add `crates/infrastructure` to `members` and an `infrastructure = { path = ... }` workspace dependency.
- `crates/level`: add `infrastructure` dep; in `spawn.rs` spawn one `storage_building(...)` at a free
  position with a starting `Inventory`; add `STORAGE_SIZE` to `level/src/constants.rs` (size is the
  level-side knob, per the existing comment in `spawn.rs`).
- `crates/app`: add `infrastructure` dep; add `InfrastructurePlugin` to the plugin tuple in `main.rs`.

## Observing changes (debug)

Add `debug_storage_input` system in the **level** crate (gameplay layer, not render): on number-key
presses, send `ResourceTransfer` messages (deposit/withdraw) to the `Storage` entity and `info!` the
resulting inventory so the change is visible in the console. Registered via `LevelPlugin`. This is
the "actually change the values" demo; a HUD text readout is out of scope for now.

## Tests

Unit tests in `infrastructure` mirroring `obstacle/src/bundle.rs` tests:
- `Inventory::add` / `remove` clamps at 0; `remove` returns actual amount removed.
- `apply` dispatches Deposit/Withdraw correctly.
- `storage_building` bundle yields `Storage` + `Solid` (via require) + `Static` + `Collider` + `Inventory`.
- Message round-trip: minimal `App` with `InfrastructurePlugin`, spawn storage, write a
  `ResourceTransfer`, run `update()`, assert `Inventory` changed.

(Per project rule: no existing tests edited/deleted.)

## Verification

1. `cd /home/isaak/RustroverProjects/pathfinding && ./bin/housekeeping.sh` — clean clippy + fmt, no warnings.
2. `cargo test -p infrastructure` — unit tests pass.
3. `cargo run -p pathfinding` — a colored square (storage) appears in the world; the player collides
   with it (it's `Solid`+`Static`). Press the debug keys; console logs show grain/coal/lumber/iron-ore
   values rising/falling and clamping at 0.
