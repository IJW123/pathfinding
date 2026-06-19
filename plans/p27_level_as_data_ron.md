# p27 — Level as data (typed structs + RON asset)

## Goal
Kill the scattered-object-data smell: one storage building's data currently lives across an inline
literal (pos), a `level::constants` const (size), and an `objects::constants` const (capacity/dock).
Replace authored object data with a single `assets/level.ron` file, loaded at startup into typed
spec structs, mapped to bundles by the existing `objects` constructors. Mirrors the in-repo `sprites`
pipeline (raw serde manifest → in-memory resource → consumed at spawn).

## Scope
**In:** the discrete authored objects — obstacles, the storage building, the carrier/player.
**Out (stays as constants):** `MAP_HALF_EXTENT` + all terrain/feature constants (`FEATURE_SEED`,
`HILL_*`, `MOUNTAIN_*`, `AUTHORED_FEATURES`) — `MAP_HALF_EXTENT` is shared with `terrain.rs`, and
terrain is a different subsystem (procedural recipe, not hand-placed objects). Terrain-as-data is a
clean follow-up, not this change. Boundary walls stay derived from `MAP_HALF_EXTENT`+`WALL_THICKNESS`
in `spawn_level` (parameterized by map size, never authored per wall — no smell there).
**Z values stay in code** — `OBSTACLE_Z` (obstacle crate), `STORAGE_Z` (logistics crate), and the
player's Z (owned by `player::bundle::player`, no `PLAYER_Z` const exists) are per-kind render
ordering, not per-instance layout; the asset carries `x, y` only. `CarrierSpec.spawn` is a `Vec2`;
`player(spawn)` applies the Z.

## Pattern (mirror `sprites`)
Two layers, exactly like `sprites::manifest` (raw, serde, tuple positions) → `sprites::catalog`
(in-memory, glam types, a `Resource`). Tuple positions in the raw layer avoid needing bevy/glam's
serde feature.

### Module layout (under existing `level/src/objects/`)
```
objects.rs          // decls only: pub mod {constants, manifest, spec, loader, storage, player};
objects/
  constants.rs      // now just: LEVEL_PATH = "assets/level.ron" (capacity/size consts GONE → RON)
  manifest.rs       // RAW serde layer, PURE: #[derive(Deserialize)] structs (tuple pos) + RawCommodity
                    //   enum + `RawCommodity::to_commodity(self) -> Commodity` (inherent method, see
                    //   decision 1). NO knowledge of the in-memory types — mirrors sprites::manifest.
  spec.rs           // IN-MEMORY: LevelSpec (Resource) + ObstacleSpec/StorageSpec/CarrierSpec, glam/
                    //   Commodity, AND the `From<Raw…>` impls (in-memory imports raw, exactly like
                    //   sprites::catalog holds `From<RawSpriteDef>`). Keeps the raw layer dependency-free.
  loader.rs         // load_level_spec (PreStartup): fs read + ron::from_str -> insert LevelSpec
  storage.rs        // storage(&StorageSpec) -> impl Bundle   (signature widened from scalars)
  player.rs         // carrier_player(&CarrierSpec) -> impl Bundle
```
**Where `From` lives — corrected from a naive mirror:** sprites puts `From<RawSpriteDef> for SpriteDef`
in `catalog.rs` (the in-memory side), *not* `manifest.rs`, so the raw serde layer imports nothing from
the in-memory layer. Put the `From<Raw…>` impls in `spec.rs` to keep that one-way dependency. Each
raw struct's `pos: (f32, f32)` maps to `Vec2`; the only non-trivial mapping is stock (see decision 1).

### In-memory spec (`spec.rs`)
```
#[derive(Resource)] struct LevelSpec { obstacles: Vec<ObstacleSpec>, storage: StorageSpec, carrier: CarrierSpec }
enum ObstacleShape { Circle, Triangle, Quad, Pentagon }   // see note: shared, not mirrored
struct ObstacleSpec { shape: ObstacleShape, pos: Vec2, rotation: f32, size: f32, pushable: bool }
struct StorageSpec  { pos: Vec2, half_extent: f32, max_volume: f32, dock_radius: f32, stock: Vec<(Commodity, u32)> }
struct CarrierSpec  { spawn: Vec2, max_weight: f32, max_volume: f32 }
```
`shape`+`size` → collider via `obstacle::shape::{circle,triangle,quad,pentagon}` at spawn.

**`ObstacleShape` is shared, not mirrored.** Unlike `Commodity` (a foreign domain type), `ObstacleShape`
is a new level-local fieldless enum — derive `Deserialize` on it once and use the *same* type in both the
raw `RawObstacleSpec` and the in-memory `ObstacleSpec`. The only field that differs between raw and spec
is `pos` (`(f32,f32)` → `Vec2`). No `RawObstacleShape`. (Mirroring is the established pattern only where a
foreign crate would otherwise need a serde dep — that's `Commodity`, not this.)

**Dropped `sprite: String` from `StorageSpec` (scope tightening).** The constructor hardcodes
`SpriteId::new("warehouse")` — a single literal, not "scattered authored data," and there's exactly one
storage. Promoting it to a stringly-typed RON field is YAGNI and weakens type safety. Leave the sprite id
in `storage()`. Reintroduce only if/when the level authors multiple storages with distinct sprites.

### Constructors become spec→bundle (consts vanish into data)
- `storage(&StorageSpec)` reads `max_volume`/`dock_radius`/`sprite`/`half_extent`/`stock` from the
  spec instead of `objects::constants` — so **`STORAGE_MAX_VOLUME`, `STORAGE_DOCK_RADIUS`,
  `CARRIER_MAX_WEIGHT`, `CARRIER_MAX_VOLUME` are deleted** (they live in `level.ron` now).
- `carrier_player(&CarrierSpec)` reads the two caps + spawn from the spec.

### `spawn_level` (Startup) reads the resource
`fn spawn_level(commands, level: Res<LevelSpec>)`:
walls loop (const-driven) → `for o in &level.obstacles { spawn static/pushable from shape+size+Z+rot }`
→ `commands.spawn(storage(&level.storage))` → `commands.spawn(carrier_player(&level.carrier))`.
Deletes the obstacle-size + storage-size imports from `level::constants`.

### `assets/level.ron` (reproduces current values exactly — identical runtime behavior)
```
(
  obstacles: [
    (shape: Circle,   pos: (250.0, 0.0),    rotation: 0.0, size: 60.0, pushable: false),
    (shape: Triangle, pos: (280.0, 160.0),  rotation: 0.6, size: 75.0, pushable: false),
    (shape: Quad,     pos: (150.0, -260.0), rotation: 0.0, size: 74.0, pushable: true),
    (shape: Pentagon, pos: (320.0, -200.0), rotation: 0.0, size: 65.0, pushable: true),
  ],
  storage: (pos: (-250.0, 200.0), half_extent: 50.0, max_volume: 20.0, dock_radius: 120.0,
            stock: [(Grain, 100), (Coal, 40), (Lumber, 60), (IronOre, 20)]),
  carrier: (spawn: (0.0, 0.0), max_weight: 2000.0, max_volume: 3.0),
)
```

### Loader: fail LOUD (diverges from sprites' warn-and-continue — justified)
`sprites` warns + leaves an empty catalog because missing textures degrade gracefully. A missing/
unparseable `level.ron` means **no world** — unplayable. So `load_level_spec` `.expect()`s with a
message naming the path (matches `spritebake`'s "fail loudly" ethos). No sensible default level.

### Plugin
`LevelPlugin`: add `.add_systems(PreStartup, load_level_spec)` (before `Startup` spawn, same ordering
`sprites` uses so the resource exists when `spawn_level` reads it). Add `serde` + `ron` to
`level/Cargo.toml`.

## Decisions to confirm
1. **Commodity in RON** — `logistics::Commodity` has no serde. Two options:
   - (a) **Mirror** a `RawCommodity` enum in `objects::manifest`, map → `Commodity` (keeps logistics
     serde-free, matches the sprites precedent). Cost: a 4-variant enum to keep in sync when a good
     is added (commodity.rs already lists touch-points; this adds one).
   - (b) **`#[derive(Deserialize)]` on `Commodity`** + serde dep on logistics. Single source of
     truth, no sync. Cost: serde in a pure domain crate.
   **Recommend (a)** — honors the established no-domain-serde pattern; the enum is tiny and stable.

   **Mechanism correction (don't write the obvious illegal impl):** `impl From<RawCommodity> for
   Commodity` does **not** compile — `Commodity` is foreign to `level` and is the `Self` type, so the
   orphan rule rejects it (same reason `impl From<MyErr> for std::io::Error` is illegal). This differs
   from sprites, where `From<RawSpriteDef> for SpriteDef` is legal because *both* types are crate-local.
   Implement the mapping as an **inherent method on the local type**:
   `impl RawCommodity { fn to_commodity(self) -> Commodity { match … } }`, called from the
   `From<RawStorageSpec>` impl in `spec.rs` (`raw.stock.into_iter().map(|(c, n)| (c.to_commodity(), n))`).
   A `#[cfg(test)]` exhaustiveness check keeps the four-variant match honest against `Commodity::ALL`.
2. **Existing integration test** `crates/level/tests/spawn_level.rs` (`spawns_expected_counts`,
   `two_obstacles_are_pushable`) calls `spawn_level`, whose signature gains `Res<LevelSpec>`. The test
   must insert a `LevelSpec` to keep running. This is a forced *adaptation*, not removed coverage —
   but per the no-silent-test-edits rule I need your OK. Proposed: build a small in-memory `LevelSpec`
   in the test (decouples it from file IO; assertions unchanged). Add a `#[cfg(test)] LevelSpec::sample()`
   helper if cleaner.

## Steps
1. `serde`/`ron` → `level/Cargo.toml`.
2. `objects/spec.rs` (in-memory types + Resource), `objects/manifest.rs` (raw + From), `objects/loader.rs`.
3. Repurpose `objects/constants.rs` → `LEVEL_PATH`; delete the four capacity consts + the four obstacle
   sizes + `STORAGE_HALF_EXTENT` from `level/constants.rs`.
4. Widen `storage`/`carrier_player` to take spec refs; update their unit tests.
5. `spawn_level` reads `Res<LevelSpec>`; `LevelPlugin` loads at PreStartup.
6. Write `assets/level.ron`.
7. Adapt `tests/spawn_level.rs` (pending decision #2).
8. `./bin/housekeeping.sh`; run the app to confirm identical world.

## Tests
- `manifest.rs`: parse a small RON string → assert it maps to the right spec (round-trips shape enum,
  tuple→Vec2, RawCommodity→Commodity).
- `spec`/constructor tests from p26 stay, retargeted to spec-ref signatures.
- Loader: not unit-tested (file IO at fixed path); covered by the app-run smoke check.
```
