# p29 — Clamp inventory seeding to capacity

## Problem
`Capacity` is enforced only at deposit time (`grantable` in the transfer/haul systems). The
seeding path bypasses it: `Inventory::from_stock` knows nothing about `Capacity`, so authored
level data can spawn a holder already over its cap. Once over-cap, `grantable` returns 0 and
silently refuses further deposits — the bad state persists undetected.

Only production over-cap site: `crates/level/src/objects/storage.rs:33` (storage seeds `stock`
against a volume cap). The carrier/player seeds `Inventory::default()` (empty), so it is safe.

## Decision
Option B — prevent over-cap at construction by clamping the seed through `grantable`. Chosen over
a dev-only debug-assert because it makes the invalid state unrepresentable rather than merely
detectable.

## Changes

### 1. `crates/logistics/src/components.rs`
Add a capacity-aware constructor that clamps each seeded commodity against the running total and
returns the rejected overflow (so goods aren't lost silently — caller decides how to surface it):

```rust
/// Build an inventory from `(commodity, count)` pairs, clamping each against `cap` as the
/// running total grows. Returns the inventory plus any rejected overflow `(commodity, dropped)`,
/// so an over-cap seed is impossible to construct yet authoring mistakes still surface.
#[must_use]
pub fn from_stock_capped(
    cap: &Capacity,
    stock: impl IntoIterator<Item = (Commodity, u32)>,
) -> (Self, Vec<(Commodity, u32)>) {
    let mut inventory = Self::default();
    let mut overflow = Vec::new();
    for (commodity, amount) in stock {
        let granted = cap.grantable(&inventory, commodity, amount);
        inventory.add(commodity, granted);
        if granted < amount {
            overflow.push((commodity, amount - granted));
        }
    }
    (inventory, overflow)
}
```
`from_stock` stays as-is (general-purpose, capacity-free path for unbounded holders/tests).

### 2. `crates/level/src/objects/storage.rs`
Bind the `Capacity` to a `let` **once**, then reuse the same value for both the seed and the
bundle field — do not construct it twice. Seed via `from_stock_capped`, `warn!` per overflow
entry with enough detail to act on (commodity, dropped count, and `spec.pos` to locate the
offending building), pass the resulting inventory into `storage_building`:

```rust
let capacity = Capacity { max_weight: None, max_volume: Some(spec.max_volume) };
let (inventory, overflow) = Inventory::from_stock_capped(&capacity, spec.stock.iter().copied());
for (commodity, dropped) in overflow {
    warn!(?commodity, dropped, ?spec.pos, "storage seed exceeds volume cap; clamped");
}
// ...storage_building(transform, …, inventory), capacity, …
```

### 3. Tests
- `components.rs`: `from_stock_capped` clamps to cap and reports overflow; under-cap seeds report
  empty overflow. Add a multi-commodity case proving the clamp tracks the *running* total across
  pairs (the second commodity sees the headroom the first already consumed), since that is the
  only behaviour `from_stock_capped` adds over `from_stock` + `grantable`.
- `storage.rs`: existing composition test still passes unchanged — its `(Grain, 100)` at
  `max_volume: 20.0` is ~3.25 m³, well under cap, so it routes through the new path untouched.
  Add one whose stock *genuinely* exceeds the cap (grain needs >~615 units at 20 m³ — pick a
  round over-shoot like 1000) and assert the spawned inventory's `total_volume() <= max_volume`.
  Use `<=` (with a small fp tolerance), not strict equality: `grantable` floors on the binding
  axis, so the result lands at-or-just-under the cap, rarely exactly on it.

## Edge cases / notes
- A commodity with `unit_volume() == 0.0` is treated as unbounded on the volume axis by
  `grantable` (the `per_unit > 0.0` guard), so it is seeded in full regardless of the volume cap.
  This is intentional and mirrors the deposit path exactly — not a regression, and not in scope to
  change here.
- Run `./bin/housekeeping.sh` after implementing; the new `#[must_use]` tuple return will trip
  clippy at every ignored call site, so confirm both call sites consume `overflow`.

## Out of scope
`from_stock` API change, runtime re-validation systems, weight-capped storage, the zero-volume
commodity behaviour noted above.

<!-- auto-reviewed -->
