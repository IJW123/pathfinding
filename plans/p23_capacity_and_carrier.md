# p23 — Capacity (weight + volume), player as carrier, and cargo handling

Capacity limits on holders, the player as a capacity-limited carrier, and real
holder-to-holder hauling (load/unload) gated by a circular dock zone. Consumes the
previously-dead `total_weight`/`total_volume`/`props` machinery from p22.

## Model (confirmed)

Two constraint axes: **weight** (kg, `total_weight()`) and **volume** (m³,
`total_volume()`, where unit volume = weight / density).

| Holder           | weight cap | volume cap | notes |
|------------------|-----------|------------|-------|
| Storage building | —         | yes        | space-limited only |
| Carrier (player) | yes       | yes        | density decides which binds first |

## Decisions taken
- Overflow on deposit: **partial fill** — move as many units as fit on *both* axes; the
  rest stays at the source (now real, because hauling has a source).
- Movement: **none new** — player already moves by keyboard; it just gains carrier comps.
- Hauling trigger: **circular dock zone** around the building; keys **9 = load-all**
  (building→player), **0 = unload-all** (player→building), only when the carrier is inside
  the zone. Existing **1–8** keep stocking the building (debug). Per action: **all goods,
  as much as fits**.
- Location: hauling lives in a new `logistics/src/cargo_handling/` module.

## 1. `Capacity` component (`logistics/src/components.rs`)
```rust
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Capacity { pub max_weight: Option<f32>, pub max_volume: Option<f32> } // None = unbounded
```
Pure, tested helper:
```rust
impl Capacity {
    /// Units of `commodity` depositable into `inv` without breaching either cap (floors the
    /// binding axis; `requested` if uncapped).
    #[must_use]
    pub fn grantable(&self, inv: &Inventory, commodity: Commodity, requested: u32) -> u32
}
```
Math: per-unit `w`/`v` from the commodity; capped-axis headroom = `floor((max-current)/per_unit)`
clamped ≥0; `min(requested, weight_count, volume_count)`.

## 2. Capacity-aware single-target transfers (`systems.rs`)
`apply_commodity_transfers` queries `(&mut Inventory, Option<&Capacity>)`. Deposit grants
`capacity.map_or(amount, |c| c.grantable(..))`; withdraw unchanged; no `Capacity` ⇒ full
deposit (today's behaviour preserved).

## 3. `cargo_handling/` module (`logistics`)
mod.rs declarations only (per repo rule). Submodules:
- `components.rs`:
  - `Carrier` — marker for a mobile cargo holder (the player now; a future vessel later). Lets
    cargo systems find the hauler **without** a `logistics → player` dep.
  - `DockZone { radius: f32 }` — circular load/unload range around a holder (the building).
- `message.rs`: `CargoHaul { source: Entity, dest: Entity }` — move *all* commodities
  source→dest, as much as fits.
- `systems.rs`:
  - `apply_cargo_hauls`: `Query<&mut Inventory>` (+ `Query<&Capacity>`). For each haul,
    `get_many_mut([source, dest])` (skip if equal/missing); per `Commodity::ALL`, move
    `min(available_at_source, grantable_at_dest)` — grantable recomputed against the *running*
    dest state so the shared weight/volume budget is respected across commodities. Remainder
    stays in source (real partial-fill).
  - `dock_haul_input`: `9`/`0` just-pressed → for each `Carrier`, find the nearest `Storage`
    whose `DockZone` contains it (xy distance ≤ radius); write `CargoHaul` in the right
    direction. Guarded with `run_if(resource_exists::<ButtonInput<KeyCode>>)` (headless-safe,
    same as the debug driver).

## 4. Plugin wiring (`logistics/src/plugin.rs`)
Register `CargoHaul`; add `dock_haul_input` (guarded) + `apply_cargo_hauls` to `Update`. Add
`pub mod cargo_handling;` to lib.rs.

## 5. Compose in `level::spawn` (knobs live here)
- Storage entity: `.insert((Capacity { max_weight: None, max_volume: Some(STORAGE_MAX_VOLUME) },
  DockZone { radius: STORAGE_DOCK_RADIUS }))`.
- Player entity: `.insert((Inventory::default(), Carrier, Capacity { max_weight:
  Some(CARRIER_MAX_WEIGHT), max_volume: Some(CARRIER_MAX_VOLUME) }))`.
- New `level/src/constants.rs`: `STORAGE_MAX_VOLUME`, `STORAGE_DOCK_RADIUS`, `CARRIER_MAX_WEIGHT`,
  `CARRIER_MAX_VOLUME`. Carrier caps small enough that loading the full building clamps (shows
  partial-fill).

`level` already depends on `logistics` + `player`; no new edges. `player` stays movement-only.

## 6. Debug driver (`logistics/src/debug.rs`)
Unchanged role: 1–8 stock the building (`With<Storage>` deposits, now capacity-clamped by the
building's volume cap). Keep as-is.

## 7. Visualize the zone (`render/src/logistics`)
`draw_dock_zones`: a `Gizmos` circle (radius = `DockZone.radius`) around each zone so the
player can see where to dock. New `DOCK_ZONE_COLOR`. Render reads logistics — correct direction.

## Tests
- `Capacity::grantable`: uncapped passthrough; weight-bound; volume-bound; over-cap ⇒ 0;
  exact-fit boundary.
- `apply_cargo_hauls`: full move when dest roomy; partial move when dest cap binds with the
  remainder left in source; shared budget across multiple commodities; source==dest no-op.
- Existing `message_mutates_target_inventory` (no `Capacity`) still passes unchanged.

## Verification
- `./bin/housekeeping.sh` clean.
- `cargo run -p pathfinding`: 1–8 fill the building (stops at its volume cap); drive the player
  into the zone circle, `9` loads until a carrier cap binds (leftover stays in building), `0`
  unloads back. `--release` still drops the debug driver.

## Out of scope (next plans)
- A standalone mobile `Vessel` entity; pathfinding around obstacles; non-debug order UI.
