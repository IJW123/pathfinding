# Plan: Generalize `infrastructure` → `logistics` (commodities any entity can hold)

## Context

The `infrastructure` crate (just built — see `plans/p20_infrastructure_storage.md`) modeled goods as
something a *storage building* owns. But holding goods is a capability many entities want: transport
trucks, warehouses, ships, depots. "Infrastructure" names buildings, not that capability. So:

- Rename the crate `infrastructure` → **`logistics`** (the domain of moving/storing goods).
- Rename the goods enum `Resource` → **`Commodity`** (the user's term).
- Keep **`Inventory`** as the general, reusable component: *any* entity with an `Inventory` holds
  commodities. The `Storage` building is just the first concrete entity that has one.
- Give each commodity a **unit weight** and a **density**, and derive **unit volume = weight / density**,
  so an `Inventory` can report total weight *and* total volume — the two limits a truck/warehouse will
  later enforce.

This is a rename + extension of the existing crate; no behavior is lost. Per project rule, use plain
`mv` (never `git mv`).

## 1. Rename the crate `infrastructure` → `logistics`

- `mv crates/infrastructure crates/logistics`.
- `crates/logistics/Cargo.toml`: `name = "logistics"`.
- Root `Cargo.toml`: member `crates/infrastructure` → `crates/logistics`; workspace dep
  `infrastructure = { path = ... }` → `logistics = { path = "crates/logistics" }`.

## 2. `Commodity` + physical properties (`logistics/src/commodity.rs`, renamed from `resource.rs`)

```rust
pub enum Commodity { Grain, Coal, Lumber, IronOre }
impl Commodity {
    pub const ALL: [Commodity; 4] = [Grain, Coal, Lumber, IronOre];
    pub fn unit_weight(self) -> f32 { /* from constants.rs */ }
    pub fn density(self)     -> f32 { /* from constants.rs */ }
    pub fn unit_volume(self) -> f32 { self.unit_weight() / self.density() } // derived
}

pub enum CommodityChange { Deposit(u32), Withdraw(u32) }   // renamed from ResourceChange
```

Weight (per unit) and density (per unit volume) are **per-commodity consts** in `constants.rs`
(project constants policy — no inline consts). Plausible, tweakable starting values, e.g.
grain `25.0 / 770.0`, coal `30.0 / 1350.0`, lumber `20.0 / 500.0`, iron ore `50.0 / 2500.0`.
`ALL` exists so `Inventory` totals can iterate commodities.

## 3. `Inventory` stays general, gains weight/volume totals (`logistics/src/components.rs`)

- Keep the `Storage` marker (`#[require(Solid)]`) and `Inventory { grain, coal, lumber, iron_ore }`.
- Existing methods retargeted to `Commodity`: `amount`, `add`, `remove`, `apply`.
- Add:
  ```rust
  pub fn total_weight(&self) -> f32  // Σ amount(c) as f32 * c.unit_weight()
  pub fn total_volume(&self) -> f32  // Σ amount(c) as f32 * c.unit_volume()
  ```
  both folding over `Commodity::ALL`.

Note: `Inventory` is deliberately decoupled from `Storage` — a truck entity later just gets an
`Inventory` (and its own marker), no storage involved.

## 4. Rename message/system/plugin (mechanical)

- `message.rs`: `ResourceTransfer` → `CommodityTransfer` (`resource` field → `commodity`).
- `systems.rs`: `apply_resource_transfers` → `apply_commodity_transfers`.
- `plugin.rs`: `InfrastructurePlugin` → `LogisticsPlugin`.
- `lib.rs`: `pub mod resource;` → `pub mod commodity;`.

## 5. Update consumers (rename imports + symbols)

- **render** (`render/src/infrastructure/` → `render/src/logistics/`): rename module dir; `lib.rs`
  `pub mod infrastructure;` → `pub mod logistics;`; `plugin.rs` import path; `Cargo.toml`
  `infrastructure` → `logistics`. `StorageRenderPlugin` / `STORAGE_COLOR` keep their names (still
  render the storage square). It keys off `Added<Storage>`, unchanged.
- **level**: `Cargo.toml` dep rename; `spawn.rs` `infrastructure::` → `logistics::` (Inventory field
  literals unchanged); `debug.rs` `Resource`→`Commodity`, `ResourceChange`→`CommodityChange`,
  `ResourceTransfer`→`CommodityTransfer`, import paths.
- **app**: `Cargo.toml` dep rename; `main.rs` `InfrastructurePlugin` → `LogisticsPlugin` + import path.

## 6. Tests

Update existing `logistics` tests for new names; add:
- `Commodity::unit_volume == unit_weight / density` for each commodity.
- `Inventory::total_weight` / `total_volume` over a known mixed stock.
Existing message-round-trip and bundle tests carry over under new names. (No tests deleted.)

## Verification

1. `cd /home/isaak/RustroverProjects/pathfinding && ./bin/housekeeping.sh` — clean clippy + fmt.
2. `cargo test -p logistics` — all unit tests pass (incl. new weight/volume/density).
3. `cargo run -p pathfinding` — storage square still renders; keys 1–8 still deposit/withdraw and the
   console logs the inventory (unchanged behavior, just renamed).

## Follow-up note

Copy this plan to `plans/p21_logistics_generalization.md` after approval (repo convention; can't write
outside the plan file while in plan mode).
