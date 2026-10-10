# p38 — Bevy 0.18 → 0.20 upgrade, BSN for the HUD

## Context

Workspace is on Bevy 0.18.1. Bevy 0.20.0 shipped 2026-10-08 (0.19.0 was 2026-06-19). Goal: jump straight to 0.20 and adopt BSN (`bsn!` scene notation, landed in 0.19, syntax stabilised in 0.20) where it actually fits.

Decisions already made with the user:
- **Target 0.20.0**, skipping 0.19 as a stopping point (BSN syntax changed between them; one migration not two).
- **BSN scope: HUD only.** It is the only scene-shaped code in the repo (`crates/hud/src/systems.rs`). Level object constructors stay `impl Bundle`; render systems keep inserting visuals via `Added<T>` (a `SceneComponent` on a logic component expanding into render parts would invert "render depends on logic"). Revisit note at the end.
- **No feathers.** Its own docs: for editors/inspectors, game UI "deliberately not intended". Headless `bevy_ui_widgets` is the game layer and is in `DefaultPlugins` now anyway.
- **rapier2d / parry2d stay at 0.33 / 0.28.** Separate pass later.

Why low risk: survey found no shaders, observers, reflection, scenes, `Resource`+`Component` dual derives, or `bevy_*` ecosystem crates. `Message`/`MessageReader`/`MessageWriter` naming is unchanged in 0.20 (verified on docs.rs). Direct hits are Cargo features, `TextFont::font_size`, and possibly sprite draw order from the new 2D backend.

## Step 1 — Cargo

`Cargo.toml` (workspace): `bevy = { version = "0.20", default-features = false, features = [...] }`.

Verified on crates.io that these 0.18 feature names still exist in 0.20: `default_app`, `2d_bevy_render`, `bevy_ui`, `bevy_ui_render`, `std`, `bevy_gilrs`, `bevy_winit`, `default_font`, `multi_threaded`, `webgl2`, `x11`, `sysinfo_plugin`, `png`.

Add:
- `bevy_scene` — the BSN crate (`scene` would also pull `bevy_world_serialization`, not needed).
- Likely needed once compiling (0.19 moved them out of `default_app`): `bevy_window`, `bevy_input_focus` (bevy_ui depends on it; `ui_api` collection bundles both). Add only what the compiler demands; keep `default-features = false`.

Nothing else in the workspace deps changes. `ron 0.8` / `serde` are app-level, independent of bevy's `ron 0.12`.

Then `cargo update -p bevy` style resolution happens via `cargo check --workspace`; fix the lock as it comes.

## Step 2 — Code migration (0.18 → 0.19 → 0.20 items that touch us)

Work by compiler errors, in this order. Expected sites:

1. **`TextFont::font_size`** is `FontSize` not `f32` (0.19). Two sites in `crates/hud/src/systems.rs:18,45`. Use `FontSize::Px(HUD_FONT_SIZE)` explicitly — 0.20 changed the default to `rem(1)`, so we keep a fixed px size on purpose. Constant stays `f32` in `crates/hud/src/constants.rs`.
2. **`Single`-skipped systems** now spawn tasks in the multithreaded executor (0.19 perf note). `update_hud_text` uses `Option<Single<..>>` so it always runs; no change. Leave the others.
3. **Sprite backend replaced** (0.20): `Sprite` gained `alpha_mode: SpriteAlphaMode` (default `Blend`) and same-Z draw order may differ. Our three `Sprite { custom_size, .. }` sites (`render/src/{player,rail,sprite}/plugin.rs`) use `..default()` so they compile; verify visually that `*_Z` constants still order correctly.
4. **Resources as components** (0.19): `world.resource::<Messages<T>>()` in tests still works. If any test uses `world.get_resource_by_id` / component-count assertions that now see resource entities, STOP and ask before editing the test.
5. **Lifecycle `Replace` → `Discard`**: grep found none. `set_if_neq` → `set_if_different`: none.
6. Anything else the compiler surfaces — check the 0.18→0.19 and 0.19→0.20 migration guides (`bevy.org/learn/migration-guides/`) per item rather than guessing.

Clippy at `-D warnings` is the gate (`bin/housekeeping.sh`); expect new deprecation warnings to count as failures.

## Step 3 — HUD on BSN

Rewrite `crates/hud/src/systems.rs::spawn_hud` as a scene function; wire it via the `SpawnListSystem` sugar.

`crates/hud/src/scene.rs` (new; `systems.rs` keeps the two update systems):

```rust
use bevy::prelude::*;
use bevy::scene::prelude::*;   // bsn!, bsn_list!, Scene, SceneList

pub fn hud() -> impl SceneList {
    bsn_list! {
        Text("x: 0 m  y: 0 m\nz: 0.0 m\nspeed: 0.0 m/s")
        TextFont { font_size: FontSize::Px(HUD_FONT_SIZE) }
        TextColor(HUD_TEXT_COLOR)
        Node { position_type: PositionType::Absolute, top: px(HUD_MARGIN_PX), right: px(HUD_MARGIN_PX) }
        HudReadout
        --
        Node {
            position_type: PositionType::Absolute,
            bottom: px(HUD_MARGIN_PX), left: px(HUD_MARGIN_PX),
            flex_direction: FlexDirection::Column, align_items: AlignItems::FlexStart,
            row_gap: px(2.0),
        }
        Children [
            Text("0 m") TextFont { font_size: FontSize::Px(HUD_FONT_SIZE) } TextColor(HUD_TEXT_COLOR) ScaleBarLabel
            --
            Node { width: px(TARGET_BAR_PX), height: px(SCALE_BAR_HEIGHT_PX) } BackgroundColor(SCALE_BAR_COLOR) ScaleBarFill
        ]
    }
}
```

`crates/hud/src/plugin.rs`: `add_systems(Startup, hud.spawn())` replacing `spawn_hud`.

Notes / things to confirm against docs.rs while implementing (syntax is 0.20's `--` / `@` form; the 0.19 `,` separator and `template_value` are deprecated):
- Marker components `HudReadout`, `ScaleBarFill`, `ScaleBarLabel` (`crates/hud/src/components.rs`) must derive `Default, Clone` to be their own templates.
- Constants used as field values: plain identifiers work; wrap in `{ }` only if the macro demands an expression.
- `row_gap: px(2.0)` — the literal `2.0` should move to `constants.rs` (constants policy) as e.g. `SCALE_BAR_ROW_GAP_PX`.
- `use bevy::scene::prelude::*` is a glob import; CLAUDE.md prefers explicit imports. Import `bsn_list`, `SceneList`, `SpawnListSystem` explicitly and see if the macro needs more.

Delete `spawn_hud`. Marker components and update systems are unchanged, so `update_hud_text` / `update_scale_bar` keep working.

## Step 4 — Housekeeping and visual check

- `./bin/housekeeping.sh` — fmt, pedantic advisory, `-D warnings` gate.
- `cargo test --workspace` — tests must pass untouched. Any test that needs an edit: pause and ask.
- `cargo run -p pathfinding` — check: HUD readout top-right, scale bar bottom-left with label, bar resizes on zoom; sprites (player, loco, warehouse) still draw above their meshes/contours; selection highlight and range disc render; no runtime warnings about missing `EmSize`/`RemSize` or scene spawn errors.

## Files

- `Cargo.toml` (workspace deps)
- `crates/hud/src/{plugin.rs, systems.rs, components.rs, constants.rs}` + new `crates/hud/src/scene.rs`, `lib.rs` module line
- Whatever else the compiler names (expected: nothing beyond `render/src/*/plugin.rs` if `Sprite` needs a field)
- `plans/p38_bevy_020_bsn_hud.md` — copy of this plan once approved (repo convention)

## Deferred / revisit

- **BSN for level constructors**: revisit when (a) the `.bsn` asset loader ships (planned, not in 0.20) and `level.ron` could become authored scenes, (b) object variants multiply so patching beats tuple composition, or (c) nested interactive UI appears. Blockers today: `RailTrack` not `Clone`/`Default`; `Inventory` computed; BSN syntax churn between releases.
- **feathers**: reconsider when an interactive panel (station cargo UI) exists; crib the BSN-widget pattern from it without depending on it.
- **rapier2d 0.36 / parry2d 0.31**: separate pass.
- `routing` crate still takes the full workspace `bevy` dep for `Vec2`; unchanged here.
