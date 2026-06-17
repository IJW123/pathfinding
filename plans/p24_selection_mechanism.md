# p24 — Mouse Selection & Arrow-Key Control

## Goal
Click an entity (player, storage, …) with the mouse to select it; arrow keys move the
*selected* entity. No pathfinding / autonomous movement. Player is just one selectable,
selected by default.

## Decisions (locked)
- **Control model:** player is a selectable. Arrow keys move `With<Selected>`. The
  controlled-movement system is generic, not player-specific.
- **Hit testing:** custom geometric test — cursor → world via `Camera::viewport_to_world_2d`,
  then parry point-in-shape against each `Selectable` entity's `Collider`. No new picking deps.
- **Highlight:** yes, render-only, reacting to the `Selected` marker.

## New crate: `selection` (world logic)
`crates/selection/` — added to workspace members + workspace deps.
Deps: `bevy`, `hitboxes_rapier` (Collider/ColliderShape + `convert::{transform_to_pose,
vec2_to_parry}`), `parry2d` (point query), `world` (HeightField + slope_speed),
`collision_rapier` (CollisionSet ordering). No `camera_main` dep — `Camera2d` is a bevy
marker, queried directly.

Files (no items in mod.rs; lib.rs only declares modules):
- `lib.rs` — `pub mod {components, systems, constants, plugin};`
- `components.rs`
  - `Selectable` — marker; entities the cursor can pick.
  - `Selected` — marker; the currently controlled entity (single-selection invariant).
- `constants.rs`
  - `CONTROL_SPEED: f32 = 300.0` (relocated from `PLAYER_SPEED`).
- `systems.rs`
  - `select_on_click` (Update): on `MouseButton::Left` just_pressed:
    - `Single<(&Camera, &GlobalTransform), With<Camera2d>>` + `Single<&Window>` (primary).
    - `window.cursor_position()` → `Option`; `None` (cursor outside window) → early return,
      do NOT deselect.
    - World point: `camera.viewport_to_world_2d(cam_transform, cursor) -> Result<Vec2, _>`;
      on `Err`, early return.
    - Query `(Entity, &Transform, &Collider), With<Selectable>`. Find the topmost hit
      (highest `Transform.translation.z`) whose collider contains the world point. Selectables
      are top-level entities → `Transform` == `GlobalTransform`, so reuse `transform_to_pose`.
    - Point test (reuse existing `hitboxes_rapier::convert`, don't hand-roll the isometry):
      `collider.shape.to_shared_shape().contains_point(&transform_to_pose(transform),
      &vec2_to_parry(world_point).into())`.
    - Found a hit → clear `Selected` from all current holders, add `Selected` to the hit.
      Empty click (no selectable under cursor) → deselect all.
  - `move_selected` (FixedUpdate, before `CollisionSet`): the generalized `move_player` —
    same arrow-key + slope logic (`HeightField::gradient` + `slope_speed_multiplier`),
    query filter `With<Selected>` instead of `With<Player>`, `CONTROL_SPEED` instead of
    `PLAYER_SPEED`. Mutates `Transform` only (matches current `move_player`).
- `plugin.rs` — `SelectionPlugin`: registers both systems on their schedules.

## Highlight render: `crates/render/src/selection/`
Render crate already depends on logic crates → add `selection` dep (render→logic, correct).
Add `mod selection;` to render's module declaration (lib.rs/main module, not mod.rs items).
- `mod.rs` — declares `components`, `constants`, `plugin` (no items).
- `components.rs` — `HighlightMarker` (child-mesh tag).
- `constants.rs` — `HIGHLIGHT_COLOR`, `HIGHLIGHT_SCALE` (~1.12), `HIGHLIGHT_Z_OFFSET`
  (small −, **child-local**: child sits behind parent so the larger scaled mesh peeks out as
  a border ring). Note: player z=1.0 keeps its highlight clear; storage z=0.5 puts its
  highlight at ~0.4, near obstacle z=0.5 — visually fine (offset is just for local layering),
  no z-fight since they're different x/y, but call it out.
- `plugin.rs` — `SelectionRenderPlugin`:
  - `attach_highlight` (Update, `Added<Selected>` with `&Collider`): spawn a child mesh
    (reuse `obstacle::mesh::shape_mesh(&collider.shape)`) scaled by `HIGHLIGHT_SCALE`,
    `HIGHLIGHT_COLOR`, local Transform z = `HIGHLIGHT_Z_OFFSET`; parent via `ChildOf`;
    tag child with `HighlightMarker`.
  - `clear_highlight` (Update, `RemovedComponents<Selected>`): for each removed entity,
    find its `HighlightMarker` children (`Query<&Children>` + filter) and despawn them.
    Guard: the entity may already be despawned (its children auto-despawn via `ChildOf`),
    so skip entities/children that no longer exist — don't despawn blindly.
- Register `SelectionRenderPlugin` in `render/src/plugin.rs`.

## Edits to existing crates
- `crates/player`: delete `systems.rs` (`move_player`) and `PLAYER_SPEED`. `PlayerPlugin`
  becomes empty → delete `plugin.rs` and its `mod plugin;`/`pub use` line in `lib.rs`.
  **`lib.rs` stays** — it still declares `components`, `bundle`, `constants`. Drop the now-unused
  `world` dep (move_player was its only user; bundle/constants don't need it) and
  `collision_rapier` dep (only `move_player` used `CollisionSet`). (`Player` marker, bundle,
  `PLAYER_SIZE`/`PLAYER_Z` stay.)
- `crates/level/src/spawn.rs`: add `Selectable` to the storage building and to the player;
  add `Selected` to the player (default selection). Add `selection` dep to level crate.
- `crates/app/src/main.rs`: drop `PlayerPlugin`, add `SelectionPlugin`.

## Risks / notes
- Storage is a `Static` Rapier body and lacks the movement-support components the player has
  (`Solid`, `MeasuredVelocity`, `PrevPosition`). `move_selected` only writes `Transform`, so
  no missing-component query mismatch — but storage motion won't feed velocity measurement or
  collision resolution the way the player's does. Acceptable for "no autonomous movement" scope;
  note it.
- Moving a `Static` body's `Transform` each tick is read by the detection-only solver. Existing
  FIXED_FIXED handling should cope, but verify at runtime that pushing storage into a dynamic
  body doesn't panic (see rapier-pipeline memory).
- Slope speed multiplier now also applies to storage etc. — consistent, harmless for now.
- `move_selected` runs every FixedUpdate tick regardless of whether anything is selected; with
  zero `Selected` entities it's a no-op empty query iter — fine.
- Single-selection only. Multi-select / drag-box is out of scope.
- Open question: should the player's initial `Selected` be re-grantable if the user deselects
  (empty click) and never re-clicks? Current design leaves nothing controllable until next
  click on a selectable — acceptable, but confirm that's the intended UX.

## Verify
`./bin/housekeeping.sh` clean, then run: click storage → highlight appears, arrow keys move
it; click player → control returns to player; click empty → deselect.
```
<!-- auto-reviewed -->
