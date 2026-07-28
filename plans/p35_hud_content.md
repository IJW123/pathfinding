# Plan: Generic HUD content panel — show selected carrier's commodity stock

## Context

The HUD top-right readout is selection-scoped: it shows the `Selected` entity's
position / elevation / speed (`crates/hud/src/systems.rs`). Entities that carry
goods (`logistics::Inventory` — storage buildings now, carriers/vessels later)
have no way to surface what they hold.

Goal: when a goods-carrying entity is selected, the HUD shows its commodity
content; the panel updates live as the inventory changes, and appears
automatically for **any** new entity that carries goods — no per-entity wiring.
"For now just commodity content" signals this should be a *general* mechanism,
so the HUD stays ignorant of commodities and future content types add a producer
in their own crate without touching the HUD.

Decisions locked in with the user:
- **Scope**: selected entity only (mirrors the existing readout).
- **Mechanism**: a generic display component, not the HUD reading `Inventory`
  directly. Keeps the dependency arrow correct (render depends on logic) and
  makes the HUD commodity-agnostic.

## Design

A passive data component `HudContent { lines: Vec<String> }` lives in a new tiny
data-only crate. Producers (world-logic crates) fill it; the HUD reads it.

```
logistics (world logic) ──writes──▶ HudContent ◀──reads── hud (render)
                                   (new data crate)
```

Flow:
1. `Inventory` `#[require(HudContent)]` → every goods-carrier auto-gets an empty
   `HudContent` at spawn. This is the foolproof bit: no spawner edits, no
   registration.
2. A logistics producer system fills each holder's `HudContent` from its
   `Inventory` on `Changed<Inventory>` (Bevy treats `Added` as `Changed`, so
   newly-spawned holders fill on their first frame — covers "automatically for
   new entities" and "update on changes" in one query).
3. The HUD reads the `Selected` entity's `HudContent` and renders its lines into
   a dedicated text node. No `Selected` → no `HudContent` → panel cleared.

The HUD never imports `logistics` or `Commodity`. Adding another content type
later = a new producer system writing into `HudContent` from its own crate.

### Naming note
The component is purely data (only a `Vec<String>`, no rendering). The user's
chosen preview named it `HudContent`; I'll keep that name. If you'd prefer the
crate/type to read fully domain-neutral (e.g. crate `describe`, type
`Description`) since the HUD is just its first consumer, say so before I start —
trivial rename, same structure.

## Changes

### 1. New crate `crates/hud_content` (data only, depends on `bevy` only)
- `Cargo.toml`: name `hud_content`, workspace package/edition, dep `bevy`.
- `src/lib.rs`: `pub mod components;` (no items in lib.rs, per project rule).
- `src/components.rs`:
  ```rust
  use bevy::prelude::Component;

  /// Pre-formatted lines describing an entity for the HUD. A producer (any
  /// world-logic crate) fills it; the HUD displays the selected entity's lines.
  /// Commodity-agnostic — the HUD never knows what produced these.
  #[derive(Component, Default)]
  pub struct HudContent {
      pub lines: Vec<String>,
  }
  ```
- A separate data-only crate is the only correct home: no existing neutral crate
  is shared by `logistics` and `hud` (logistics→hitboxes_rapier, hud→selection/
  world/motion), and putting it in `hud` would invert the render→logic arrow.
  The tiny-crate cost is the price of the dependency rule, not over-engineering.
- Root `Cargo.toml`: add `"crates/hud_content"` to `members` and
  `hud_content = { path = "crates/hud_content" }` to `[workspace.dependencies]`.

### 2. `logistics` — produce content from inventory
- `crates/logistics/Cargo.toml`: add `hud_content = { workspace = true }`.
- `crates/logistics/src/components.rs`: add `#[require(HudContent)]` to the
  `Inventory` struct (import `hud_content::components::HudContent`).
- New module `crates/logistics/src/readout.rs` with the producer system, e.g.
  `sync_inventory_hud_content`:
  ```rust
  Query<(&mut HudContent, &Inventory), Changed<Inventory>>
  ```
  Rebuild `lines` from non-zero commodities, reusing `Commodity::ALL` and
  `inv.amount(c)` (and the `{c:?}` Debug formatting already used in
  `debug.rs::log_inventory_changes`), e.g. `format!("{c:?}: {}", inv.amount(c))`.
  One line per held good, e.g. `"Grain: 30"`; empty stock → empty `lines` (HUD
  shows nothing). Register the module in `lib.rs` and the system in
  `LogisticsPlugin` (`Update`).
- Ordering: do **not** chain the producer after `apply_commodity_transfers` /
  `apply_cargo_hauls`. `Changed<Inventory>` persists until the producer's query
  observes it, so a mutation is never missed — at worst the panel lags one frame
  if the producer ran before the mutation. One-frame lag on a HUD is invisible;
  chaining it in just adds coupling for nothing.

### 3. `hud` — consume and render
- `crates/hud/Cargo.toml`: add `hud_content = { workspace = true }`.
- `crates/hud/src/components.rs`: add marker `HudContentPanel` for the text node.
- `crates/hud/src/systems.rs`:
  - `spawn_hud`: spawn a `Text` node (top-left, away from the top-right readout
    and bottom-left scale bar) tagged `HudContentPanel`, styled with the existing
    `HUD_FONT_SIZE` / `HUD_TEXT_COLOR`.
  - New `update_hud_content` system:
    ```rust
    selected: Option<Single<&HudContent, With<Selected>>>,
    mut panel: Single<&mut Text, With<HudContentPanel>>,
    ```
    Build the target string (`lines.join("\n")`, or empty when nothing selected /
    selection has no `HudContent`) and write it **only when it differs** from the
    current `panel.0` — `if panel.0 != next { panel.0 = next; }`. Writing every
    frame dirties the `Text` and forces a UI relayout each frame even when the
    stock is static; the `!=` guard sidesteps that. (`update_hud_text` /
    `update_scale_bar` write unconditionally today — same latent cost; not this
    plan's job to fix, but don't copy the pattern into the new system.)
    Must run each frame, not gated on `Changed`: selection can change without the
    inventory changing. `Single<&mut Text, With<HudContentPanel>>` is disjoint
    from the `HudReadout` text node via the marker, and `Single` silently skips
    the system until `spawn_hud` has run — both fine.
- `crates/hud/src/plugin.rs`: add `update_hud_content` to the `Update` tuple.

## Files touched
- New: `crates/hud_content/{Cargo.toml,src/lib.rs,src/components.rs}`
- New: `crates/logistics/src/readout.rs`
- Edit: root `Cargo.toml`, `crates/logistics/Cargo.toml`,
  `crates/logistics/src/{components.rs,lib.rs,plugin.rs}`,
  `crates/hud/Cargo.toml`, `crates/hud/src/{components.rs,systems.rs,plugin.rs}`

## Notes / trade-offs
- `Inventory` is `Copy`; `HudContent` holds a `Vec` so is not `Copy`. They are
  separate components, so this doesn't affect `Inventory`'s derives.
- Producer writes only on `Changed<Inventory>` — no per-frame string churn for
  static stock. HUD's per-frame work is a single small `join`.
- HUD has no `logistics` dependency; dependency arrows stay render→logic.
- Behaviour note: this panel clears on deselect, whereas the existing
  `update_hud_text` readout goes stale (never cleared). The two will visibly
  diverge. Intentional here; flagging the inconsistency, not expanding scope to
  fix the readout.

## Verification
- `cd /home/isaak/RustroverProjects/pathfinding && ./bin/housekeeping.sh` —
  clean clippy + fmt, no warnings.
- `cargo test -p logistics` — existing inventory/transfer tests still pass; add a
  headless test that spawning a `storage_building` yields a `HudContent` whose
  `lines` reflect the seeded stock after one `app.update()` (proves the
  `#[require]` + producer wiring, no rendering needed). Seed with
  `Inventory::from_stock([(Commodity::Grain, 30)])` and assert
  `lines == ["Grain: 30"]`. Bare `App::new() + LogisticsPlugin` suffices — the
  producer's query runs under `app.update()`; the input-gated debug/haul systems
  skip without `ButtonInput`, so no `MinimalPlugins` needed. `#[require]` fires
  on entity insertion (the bundle), so no type registration is required.
- `cargo run -p pathfinding`: select a storage building → panel shows its stock;
  drive the debug keys (`1`–`8` deposit/withdraw) → panel updates live; deselect
  → panel clears; select the player (no inventory) → panel stays empty.

<!-- auto-reviewed -->
