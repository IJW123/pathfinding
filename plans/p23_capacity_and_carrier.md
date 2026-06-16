# p23 — Inventory capacity (weight + volume) and the player as carrier

Introduce capacity limits on things that hold goods, and make the player a carrier.
Consumes the previously-dead `total_weight`/`total_volume`/`props` machinery from p22.

## Model (confirmed)

A holder is constrained on up to two axes:
- **weight** (mass, kg) — `total_weight()`
- **volume** (space, m³) — `total_volume()`, where each unit's volume = weight / density

| Holder            | weight cap | volume cap | why                                   |
|-------------------|-----------|------------|----------------------------------------|
| Storage building  | —         | yes        | space-limited; weight irrelevant on the ground |
| Carrier (player)  | yes       | yes        | a vehicle is mass- *and* space-limited; density decides which binds first |

Density "matters" precisely because it converts a count into volume: a dense good (iron
ore) hits the **weight** cap first; a bulky low-density good (lumber) hits the **volume**
cap first.

## Decisions taken
- Overflow on deposit: **partial fill to cap** — deposit as many units as fit on *both*
  axes, leave the rest unmoved (not silently discarded).
- Movement: **none new**. The player already moves by keyboard; it just gains carrier
  components. A standalone mobile vessel + pathfinding is a later plan.

## 1. `Capacity` component (`logistics`)

New component in `components.rs` (alongside `Inventory`/`Storage`):
```rust
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Capacity {
    pub max_weight: Option<f32>, // None = unbounded on this axis
    pub max_volume: Option<f32>,
}
```
`Option` per axis keeps it one general component: building sets `max_volume` only, carrier
sets both. `None` = no limit (preserves today's uncapped behaviour for holders without a
`Capacity`).

Pure grant helper (testable, no ECS):
```rust
impl Capacity {
    /// Units of `commodity` that may be deposited into `inv` without breaching either cap.
    /// Floors on the binding axis; returns `requested` if uncapped.
    #[must_use]
    pub fn grantable(&self, inv: &Inventory, commodity: Commodity, requested: u32) -> u32 { ... }
}
```
Math: per-unit `w = commodity.unit_weight()`, `v = commodity.unit_volume()`; headroom count
on each capped axis = `floor((max - current)/per_unit)` clamped ≥ 0; `grantable = min(requested,
weight_count, volume_count)`. `unit_weight`/`unit_volume` are > 0, so no div-by-zero.

## 2. Capacity-aware transfers (`systems.rs`)

`apply_commodity_transfers` queries `(&mut Inventory, Option<&Capacity>)`:
- **Deposit**: `granted = capacity.map_or(amount, |c| c.grantable(&inv, commodity, amount))`,
  then `inv.add(commodity, granted)`. No `Capacity` ⇒ full deposit (unchanged behaviour).
- **Withdraw**: unchanged (already clamps at 0; caps are upper bounds only).
- Partial-fill visibility: in debug, `log` when `granted < amount` (shortfall). No new
  outcome message yet — a real remainder only exists once load/unload has a *source*
  inventory (next plan); deposits today are conjured by the debug driver.

## 3. Attach capacity where the knobs live (`level`)

`Capacity` magnitudes are per-instance level knobs (like size/position), so they're inserted
at spawn in `level::spawn` — NOT baked into `storage_building`'s signature. This keeps the
bundle (and its tests) untouched and mirrors how the player gets its components.

- Storage entity: `.insert(Capacity { max_weight: None, max_volume: Some(STORAGE_MAX_VOLUME) })`.
- Player entity: spawn already returns `EntityCommands`; chain
  `.insert((Inventory::default(), Capacity { max_weight: Some(CARRIER_MAX_WEIGHT),
  max_volume: Some(CARRIER_MAX_VOLUME) }))`.
- New constants in `level/src/constants.rs`: `STORAGE_MAX_VOLUME`, `CARRIER_MAX_WEIGHT`,
  `CARRIER_MAX_VOLUME`.

`level` already depends on both `logistics` and `player`; no new crate edges. `player` stays
movement-only (no logistics dep); `logistics` stays cargo-only.

## 4. Debug driver retarget (`logistics/src/debug.rs`)

Currently deposits into `With<Storage>`. Retarget to `With<Inventory>` so the player
(carrier) also receives — lets you watch the weight cap clamp on the player and the volume
cap clamp on the building from the same keypresses. Dev-only, already gated.

## Tests
- `Capacity::grantable`: uncapped passthrough; weight-bound; volume-bound; already-over-cap
  ⇒ 0; exact-fit boundary.
- transfer partial-fill: deposit exceeding a cap lands exactly the fitted amount, not more.
- Existing `message_mutates_target_inventory` (no `Capacity` on the test entity) still passes
  unchanged — proves the uncapped path is preserved.

## Verification
- `./bin/housekeeping.sh` clean.
- `cargo run -p pathfinding`: drive the player, press 1–8; player stock stops climbing at its
  weight/volume cap, building stock stops at its volume cap. `--release` still drops the debug
  driver.

## Out of scope (next plans)
- Load/unload *between* a carrier and a building (transfers with a source + the real remainder).
- A standalone mobile `Vessel` entity and pathfinding around obstacles.
