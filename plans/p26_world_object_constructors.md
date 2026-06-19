# p26 — World object constructors

## Problem
`level/src/spawn.rs::spawn_level` does two jobs: level *layout* (positions/sizes) and entity
*composition* (assembling the full component set per object). The composition is the bloat —
`spawn_level` takes clean bundle fns (`storage_building`, `player`) then bolts on piles of
components inline via `.insert((...))`:
- Storage: `Capacity`, `DockZone`, `SpriteRef`, `Selectable`, `MeasuredVelocity`, `PrevPosition`
  (lines 64–84, ~30 lines — "the nearly 30 line thing").
- Player: `Inventory`, `Carrier`, `Capacity`, `Selectable`, `Selected`.

"What an X entity is made of" is not layout. It should live in one place per object, not inline.

## Approach (chosen: composition root in `level`)
`level` already depends on logistics/obstacle/player/selection/motion/sprites, so it is the natural
composition root. No domain crate learns about cross-crate concerns (selection/motion/sprites).
Rejected enriching domain `bundle.rs` — that would force `logistics` to depend on
selection/motion/sprites, violating the one-way dependency rule.

### Module layout (2018-style module file, NOT mod.rs — respects the no-fns-in-mod.rs rule)
```
level/src/
  objects.rs          // declarations only: `pub mod constants; pub mod storage; pub mod player;`
  objects/
    constants.rs      // composition magnitudes both constructors need (pub const only)
    storage.rs        // pub fn storage(transform, half_extent, stock) -> impl Bundle
    player.rs         // pub fn carrier_player(spawn: Vec2) -> impl Bundle
```
Add `pub mod objects;` to `lib.rs`.

### Constructors return complete bundles
Each returns the FULL entity as one (nested) tuple bundle. `spawn_level` becomes spawn-only.

- `objects/storage.rs`
  `pub fn storage(transform: Transform, half_extent: f32, stock: Inventory) -> impl Bundle`
  → `(storage_building(transform, Vec2::splat(half_extent), stock), Capacity{..}, DockZone{..},
      SpriteRef{..}, Selectable, MeasuredVelocity::default(), PrevPosition(transform.xy))`
  Seeds `PrevPosition` from `transform.translation.xy()` (kills the dependence on a separate
  `storage_pos` var). `half_extent` is passed in, NOT owned: size is a level-side layout knob (see
  `constants.rs` doc and the obstacle precedent `circle(CIRCLE_RADIUS)` in the same file). One scalar
  feeds both the square collider (`splat`) and the sprite `world_size` (`half_extent * 2.0`), keeping
  the square assumption in one spot. Pulls `STORAGE_MAX_VOLUME` / `STORAGE_DOCK_RADIUS` from
  `objects::constants`. The rationale comments currently inline in `spawn_level` (sprite-skins-collider,
  PrevPosition-seeded-to-spawn) travel here with the code.
  - `Storage` requires only `Solid` (verified), so `MeasuredVelocity` + `PrevPosition` are genuinely
    added by hand — they are NOT free via `#[require]` the way they are for `Player`.

- `objects/player.rs`
  `pub fn carrier_player(spawn: Vec2) -> impl Bundle`
  → `(player(spawn), Inventory::default(), Carrier, Capacity{..}, Selectable, Selected)`
  - `player(spawn)` already ships `PrevPosition(spawn)`, and `Player` requires `MeasuredVelocity` — so
    this must NOT re-add either (verified against `player/src/bundle.rs` + `components.rs`). Pulls
    `CARRIER_MAX_WEIGHT` / `CARRIER_MAX_VOLUME` from `objects::constants`.

### What stays in spawn_level (it IS the layout)
- `boundary_walls(...)` loop.
- The four obstacles — already pure one-line `static_obstacle/pushable_obstacle(transform, shape)`
  bundles. Transform+shape is exactly the layout/silhouette split the file's doc comment describes;
  no composition bloat to extract. **Left as-is on purpose.**
- Per-instance transforms, the storage **size** (`STORAGE_HALF_EXTENT`), and the storage starting
  stock — all layout decisions, passed into the constructors. `STORAGE_HALF_EXTENT` therefore STAYS
  in `level/src/constants.rs` (it's the size knob `spawn_level` uses), as do `MAP_HALF_EXTENT` and the
  obstacle sizes.

### Constants migration (per the per-module constants.rs policy)
Composition magnitudes are *moved* (not copied — avoid a stale duplicate) from `level/src/constants.rs`
into the new `level/src/objects/constants.rs`:
`STORAGE_MAX_VOLUME`, `STORAGE_DOCK_RADIUS`, `CARRIER_MAX_WEIGHT`, `CARRIER_MAX_VOLUME`. One shared
`objects/constants.rs` for both submodules (KISS — don't fragment four consts across two files). Their
existing doc comments ("logistics owns the model; level sets the magnitudes") move with them.

### Result
`spawn_level` shrinks to: walls loop, 4 obstacle spawns,
`commands.spawn(storage(tf, STORAGE_HALF_EXTENT, stock))`, `commands.spawn(carrier_player(Vec2::ZERO))`.
Its import list drops the motion/selection/capacity/dock/sprite imports and the composition constants
(they move to the objects modules); it keeps `STORAGE_HALF_EXTENT`, `MAP_HALF_EXTENT`, obstacle sizes,
`Inventory`/`Commodity` (the seed stock), and the obstacle/wall imports.

## Steps
1. Add `objects.rs` + `objects/constants.rs` + `objects/storage.rs` + `objects/player.rs`; wire
   `pub mod objects;` in lib.rs.
2. Move the four composition constants from `level/src/constants.rs` into `objects/constants.rs`.
3. Move composition into the two constructors.
4. Add `#[cfg(test)]` composition tests mirroring `bundle.rs`'s style: spawn each constructor into a
   bare `World`, assert the full added component set is present (`Capacity`, `DockZone`, `SpriteRef`,
   `Selectable`, `MeasuredVelocity`, `PrevPosition` for storage; `Inventory`, `Carrier`, `Capacity`,
   `Selectable`, `Selected` for the carrier) and that `PrevPosition` is seeded to the transform/spawn.
   This is the only test coverage of the composition that's being extracted — don't ship it untested.
5. Rewrite `spawn_level` to spawn-only; prune its imports.
6. `./bin/housekeeping.sh`, fix warnings.

## Resolved
- **Starting stock**: stays as the literal `Inventory::from_stock([...])` in `spawn_level` — it's a
  layout decision, consistent with passing `transform`/`half_extent` in. `storage()` takes it as a
  param; it does NOT default the stock internally.
- **Size**: passed in as `half_extent: f32`, not owned by `storage()` (see constructor note above).

<!-- auto-reviewed -->
