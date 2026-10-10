# p37 — Bevy 0.19 upgrade + BSN adoption

## Context

- Current: bevy 0.18.1. Latest stable: **0.19.1** (0.20.0-rc.2 exists, not stable). rustc 1.99 ≥ MSRV 1.95.
- BSN (`bsn!`, `bsn_list!`, `impl Scene`, `spawn_scene`) shipped in 0.19 as new `bevy_scene`. Old `bevy_scene` → `bevy_world_serialization` (we don't use it).
- 0.19 BSN is code-only: no `.bsn` asset loader yet. `assets/level.ron` stays as is.
- rapier2d/parry2d are bevy-independent (`collision_rapier` deps: bevy + rapier2d + hitboxes, no glue crate) → not
  touched (0.36 exists; separate plan if wanted).

Three phases, each ends with green housekeeping + tests. Each lands (user commits) before the next starts — never
mix an engine bump with a spawn-API rewrite, and never either with an unrelated refactor.

## Phase 0 — Clean baseline (prerequisite)

The tree is currently mid-refactor (`hitboxes_rapier` → `hitboxes`, `motion` → `measurement`, ~50 dirty files,
`Cargo.lock` modified). An engine bump on top of that makes every breakage ambiguous.

1. User finishes + commits the in-flight refactor. Upgrade does not start on a dirty tree.
2. On 0.18.1, record the baseline: housekeeping green, `cargo test --workspace` green, clean-build time
   (`cargo clean && cargo build -p pathfinding --timings`), and screenshots of HUD readout, scale bar, range
   disc, selection highlight, contours at default zoom. Phase 1 step 5 compares against these — "eyeball" with
   nothing to compare to is not a check.

## Phase 1 — Upgrade to 0.19.1 (no behaviour change)

1. `Cargo.toml`: `bevy = "0.19"`, then `cargo update -p bevy`. All 13 feature flags we use still exist in 0.19
   (verified via `cargo info`). No BSN feature yet — that's Phase 2.
2. Known breakage in our code (from migration guide vs grep):
   - `crates/hud/src/systems.rs` ×2: `TextFont.font_size: f32` → `FontSize::Px(HUD_FONT_SIZE)`. Constant stays `f32`.
     (Only `TextFont` use in the tree; no `Text2d`.)
3. Checked, expected no-ops (none found in tree): `#[derive(Component, Resource)]` on one type,
   `Query<EntityMut/EntityRef>` broad queries, non-send resources, `Assets::get_mut` (our `get_mut` hits are
   `Query`/rapier), `ExecutorKind`, `InputFocus`, `Image::pixel_bytes`, old scene types, `rand`.
4. Compile → fix whatever the guide/grep missed. Likely suspects: `bevy::sprite_render::AlphaMode2d` path
   (`render/range`), `Mesh2d`/`ColorMaterial` imports, `bevy::camera::ScalingMode` + `Projection`
   (`camera_main`), `AssetPlugin` fields (`app/main.rs`).
   - Allowed here: mechanical renames / import-path moves / signature updates.
   - **Stop and discuss**: anything that changes behaviour, system ordering, or a type's design.
   - Deprecations get fixed, not silenced — no `#[allow(deprecated)]`/`#[expect(deprecated)]` (NO QUICK FIXES).
5. Manual run (`cargo run -p pathfinding`) vs Phase 0 screenshots: text moved cosmic-text → parley, so compare HUD
   readout + scale-bar label for size/baseline shifts. Also contour, range disc (blend), selection highlight,
   sprites. A visible text shift is reported, not "fixed" by tweaking `HUD_FONT_SIZE` — that's a decision.
6. Housekeeping + `cargo test --workspace` (housekeeping is clippy+fmt only; tests are a separate command).
   Headless tests (`level/tests/spawn_level.rs`, `station/tests/haul.rs`, in-crate bare-`World` tests) are the
   canary for ECS-level changes. **If any test needs an edit → stop and ask.**
7. Re-measure clean-build time; note delta.

## Phase 2 — BSN

Scope rule: BSN is used in presentation crates only (`hud`, later maybe `render`). World-logic crates keep plain
bundles (see 2d). The feature is workspace-wide, so this is convention, not compiler-enforced.

### 2.0 Spike (throwaway, not committed) — before anything else in Phase 2

Everything below is written from release notes, not compiled. One scratch example answers, against real 0.19.1:
- Feature name: does `bevy_scene` alone give `bsn!` without `bevy_world_serialization`? Is its plugin part of
  `DefaultPlugins` once enabled, or must `HudPlugin` add it?
- Exact syntax: `px(..)`, const paths as field values (bare `HUD_FONT_SIZE` vs `{HUD_FONT_SIZE}`), enum variants
  (`FontSize::Px(..)`, `PositionType::Absolute`), `Children [ .. ]`, marker components.
- Spawn at `Startup`: `scene_fn.spawn()` as a system, or a one-line `commands.spawn_scene(hud())` system?
- Timing: are entities present the same frame, or resolved later? (Update systems use `Single<..>`, which skips
  when absent — deferred spawn is tolerated, but confirm no warning spam / panics.)
- For 2c only: is there a first-class "apply scene to existing entity" / "spawn scene as child of entity" call?

Outcome updates this plan (names, go/no-go for 2c) before 2a starts. If `bevy_scene` drags in something we don't
want, or the HUD needs workarounds → stop; Phase 2 is dropped and Phase 1 stands alone.

### 2a. HUD (clear win)

`hud/src/systems.rs::spawn_hud` is the one hierarchical, static, UI spawn — exactly BSN's sweet spot.
- Add `bevy_scene` to workspace bevy features (not `scene` — that also pulls `bevy_world_serialization`).
- New `crates/hud/src/scene.rs`: `hud_readout() -> impl Scene`, `scale_bar() -> impl Scene` (`Children [ label, fill ]`), `hud() -> impl SceneList`.
  `lib.rs` gets the `pub mod scene;` line only.
- `Val::Px(X)` → `px(X)`, drop `..default()` (BSN patches fields), drop `with_children` closure.
- Plugin: spawn `hud()` at `Startup` in whatever form the spike confirmed; delete `spawn_hud`.
- Constants policy: the inline `row_gap: Val::Px(2.0)` moves to `hud/constants.rs` as part of the rewrite (that
  line is being retyped anyway). No other values change.
- Update systems (`update_hud_text`, `update_scale_bar`) untouched — they query marker components. `systems.rs`
  keeps only the update systems.
- Verify: same screenshots as Phase 1; readout updates on selection; scale bar tracks zoom. `hud` has no tests —
  not adding any here (out of scope); the manual check is the gate.

### 2b. Camera — recommend skip

`camera_main::spawn_camera` is one flat entity whose only non-default field is built with
`..OrthographicProjection::default_2d()` — a non-`Default` base. BSN patches from the type's default, so the whole
projection becomes one `{expr}` escape wrapping today's code. That's the 2d argument verbatim; "consistency" isn't
a reason to convert something BSN makes no simpler. Leave as is.

### 2c. Render attach systems — deferred, spike-gated

`render/{selection,range,obstacle,station,rail,logistics}/plugin.rs` each take
`ResMut<Assets<Mesh>>` + `ResMut<Assets<ColorMaterial>>` just to `add()` inline. BSN's `asset_value(..)`
template would remove both params. (`elevation/contour/render.rs` also takes them but isn't an attach system —
excluded.)

Honest cost/benefit: every value here is runtime (`shape_mesh(&collider.shape)`, `ring.radius`,
`RANGE_DISC_Z - transform.translation.z`), so each field is an `{expr}` escape; the only gain is two system
params per system. And they attach to an *existing* entity (`with_child` / `insert`) from an `Added<T>` query.
Only proceed if spike 2.0 shows a first-class apply-to-entity / spawn-as-child API **and** a converted
`attach_range_disc` reads shorter than the current one. Otherwise drop (NO QUICK FIXES). If it proceeds: its own
follow-up plan, one module first — not a 6-module fan-out here.

### 2d. World-logic constructors (`*/bundle.rs`, `level/objects/*`) — recommend NOT converting

Pushing back here. Reasons:
- They're flat, single-entity, parameterised by runtime values (computed `Collider`, capped `Inventory`,
  track-sampled `Transform`). BSN adds nothing over a tuple; every field becomes a `{expr}` escape.
- Scenes are reusable templates → values need `Clone`/`FromTemplate`. `RailTrack` is deliberately not
  `Clone` (`rail_track` consumes it; see `rail/src/spawn.rs` doc). Would force a design change for syntax.
- `spawn_rail_line` reserves an entity id then inserts — imperative by necessity, not a scene.
- **~13 `world.spawn(..)` sites across 10 test-bearing files** (by grep) run on a bare `World`/headless `App`.
  `impl Scene` return types force editing every one (→ `spawn_scene`, possibly needing scene/asset plugins in a
  bare World). High churn, zero coverage gain, and tests are ask-first.
- `#[require(..)]` already gives the spawn-time guarantees `SceneComponent` advertises, for our flat entities.
- Keeps world logic free of a first-release, UI-motivated API — world crates stay testable with a bare `World`.

Revisit when the `.bsn` asset loader lands (0.20+?): then `level.ron` → `.bsn` is a real architectural win
and worth the test rewrite in one go.

## Decisions needed

1. **Target version**: 0.19.1 stable (recommended) vs 0.20.0-rc.2 (newer BSN, RC churn, redo on final).
2. **BSN scope**: **2a only (recommended)** / +2b (pure preference, no simplification) / +2c as a follow-up plan
   if the spike passes / +2d (not recommended; means editing ~13 test spawn sites).
3. HUD scene fns in new `hud/src/scene.rs` (recommended, mirrors `bundle.rs` convention) vs keep in `systems.rs`.
4. Is Phase 2 worth it at all for one HUD spawn? It's a ~40-line readability win plus a new feature flag and a
   first-release macro. Fine as a low-risk way to learn BSN before `.bsn` assets; not fine if the spike shows
   friction. Phase 1 is valuable on its own either way.

## Risks / unknowns

- BSN is first-release: macro error messages + rustfmt/RustRover support inside `bsn!` are rough. `cargo fmt`
  won't format macro bodies → `scene.rs` formatting is manual; keep it small.
- Compile time: `bevy_scene` + macros add some; measured at Phase 0 / 1 / 2a.
- BSN syntax details above are from the 0.19 release notes, not yet compiled against — exact names
  (`px`, `.spawn()`, `asset_value`) are verified in spike 2.0 *before* any committed code, not during 2a.
- Parley text metrics may shift HUD layout slightly (Phase 1 step 5).
- Scene spawn may resolve later than `Startup` commands did; `Single`-based HUD systems tolerate it — confirmed
  in spike.

## Out of scope

rapier/parry bump, `level.ron` → `.bsn`, feathers widgets, 0.20, new HUD tests, deduping per-entity
`ColorMaterial`s in render attach systems, the in-flight `hitboxes`/`measurement` refactor itself.

## Review notes (what changed vs first draft)

- Added Phase 0: dirty tree + no visual/build baseline made Phase 1's "no behaviour change" unverifiable.
- API verification moved from "during 2a" to an upfront throwaway spike; 2c's spike folded into it.
- 2b flipped to skip: `default_2d()` base defeats BSN patching — same reasoning the draft used against 2d.
- 2c demoted to a spike-gated follow-up plan; its real gain is only two system params.
- Recommended scope now 2a only. Test-site count corrected (~13 across 10 files, by grep).
- Added stop rules for Phase 1 fixes, constants-policy fix for the inline `2.0`, BSN-in-presentation-only rule.

<!-- auto-reviewed -->

