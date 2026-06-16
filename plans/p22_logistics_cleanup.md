# p22 — Logistics cleanup (commodity props + debug gating/relocation + index churn)

Follow-up to p20/p21. Four items from the diff review, no behaviour change to the
shipped game (debug controls become dev-only, commodity numbers unchanged).

## 1. Single source of truth for commodity properties (elegance)

**Problem:** `unit_weight()` and `density()` are two parallel 4-arm matches; adding a
*property* (e.g. spoilage) means a whole new match fn. Reading "everything about grain"
means hopping arms.

**Change** (`crates/logistics/src/commodity.rs`):
- Add a grouping record:
  ```rust
  pub struct CommodityProps { pub unit_weight: f32, pub density: f32 }
  ```
- One `const fn props(self) -> CommodityProps` match, **referencing the existing
  `constants.rs` consts** (NOT inline literals — honours the constants policy):
  ```rust
  Commodity::Grain => CommodityProps { unit_weight: GRAIN_UNIT_WEIGHT, density: GRAIN_DENSITY },
  ```
- `unit_weight`/`density`/`unit_volume` become thin delegators to `props()`.
- `constants.rs` is untouched (numbers stay there).

**Net:** physical data per commodity lives in one arm; new property = one struct field +
one const per commodity, referenced in the single `props()` arm.

## 2. Fix the misleading "extend in one place" comments (concern #3)

`commodity.rs` / `components.rs` claim new goods extend "here and through `Inventory`
only". Reword to the truth: a new good touches the `Commodity` variant + its `props()`
arm, plus the `Inventory` field + `slot_mut`/`amount` mapping. No overselling.

(We are NOT array-backing `Inventory` — named fields + weight/density stay. Decision per
discussion: only worth it past ~8 goods.)

## 3. Gate debug systems behind `#[cfg(debug_assertions)]` (concern #1)

`debug_storage_input` + `log_inventory_changes` currently run in **every** build,
including `--release`, despite "Debug-only" docs. Gate them so they compile out of
release. Mechanism: `#[cfg(debug_assertions)]` (zero config, off in `--release`) rather
than a cargo feature (keeps it simple).

## 4. Relocate the debug driver out of `level` (concern #2)

`level` populates the world; keyboard-input → message-writing is not its job. Move the
driver next to the API it exercises.

**Target: `logistics` crate**, gated.
- Create `crates/logistics/src/debug.rs` with the two systems (imports become `crate::`
  instead of `logistics::`).
- `logistics/src/lib.rs`: `#[cfg(debug_assertions)] pub mod debug;`
- `logistics/src/plugin.rs`: under `#[cfg(debug_assertions)]`, add the two systems to
  `Update`. Update the plugin doc (currently "Content only — no spawning, no rendering")
  to note the dev-only debug driver.
- Delete `crates/level/src/debug.rs` (plain `rm`).
- `level/src/lib.rs`: drop `pub mod debug;`.
- `level/src/plugin.rs`: revert the `Update` line to just `spawn_level`; drop the debug
  imports.
- `level` keeps its `logistics` dep (still spawns storage in `spawn.rs`).

**Alternative considered:** put it in `app` (the composition crate). Rejected — less
cohesive than co-locating with the message API; `app` is thin glue.

## 5. Stale index entry: `hill.rs` (concern #4) — USER ACTION

`hill.rs` was refactored into `feature.rs`; the staged "new file" is a dead `git add`.
File is already absent from disk and HEAD. **You** unstage it (I don't run VCS):
```
git restore --staged crates/world/src/elevation/generation/hill.rs
```

## Verification
- `./bin/housekeeping.sh` clean (clippy gate).
- Logistics tests still pass; debug systems compile under dev, vanish under `--release`
  (`cargo build --release` sanity check).
- `level` no longer references `debug`.

## Out of scope
- Array-backing `Inventory` (elegance #2) — deferred.
- Any new commodities or gameplay drivers.
