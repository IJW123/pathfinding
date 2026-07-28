# Plan: Stations as the universal loading point + dock-zone cleanup

## Context

`p33_rail_stations.md` shipped the station broker but deliberately deferred "folding the direct
carrier-dock path into the station model." This is that cleanup, plus what the user clarified:

- **Warehouses have no dock-zone.** The carrier↔warehouse path (`DockZone`, `dock_haul_input`,
  keys `9`/`0`) goes away entirely.
- **Stations are the only loading point.** A station detects what sits in its radius; when a
  `Carrier` (the player *or* a train) and one or more `Storage` warehouses are both in range, the
  load/unload keys broker between them. The mover side is no longer locomotive-specific — any
  `Carrier` qualifies.
- **No hardcoded routes.** The `find_path`-routed demo (`station_demo.rs`) is deleted. Rail lines,
  stations, and warehouses are all plain authored objects in `level.ron`; there can be many of each
  on different lines.

Net effect: one cargo-transfer UX (stations, keys `L`/`U`) for both player and trains; `level`
sheds its `routing` dependency; `station` sheds its `rail` dependency.

## Decisions settled with the user

- Mover identity = the existing `logistics::Carrier` marker (repurposed: "mobile cargo holder that
  loads/unloads at stations"). Reused by both the player and the new cargo train, so the broker
  query is `(With<Carrier>, With<Selected>)` — no `rail` coupling.
- `level.ron` schema pluralizes: `storages: [...]`, `rails: [...]`, new `stations: [...]`.
- Authored starter scene proves the feature without any routing (concrete coords below).

## Changes

### 1. `logistics` — remove the dock path, keep `Carrier`
- `cargo_handling/components.rs`: delete `DockZone`. Keep `Carrier`; rewrite its doc to "mobile
  cargo holder that loads/unloads at a station in range."
- `cargo_handling/systems.rs`: delete `dock_haul_input` and its now-unused `DockZone`/`Carrier`
  imports. `apply_cargo_hauls` is unchanged.
- `plugin.rs`: drop `dock_haul_input` from the `Update` chain; register `apply_cargo_hauls` alone
  (no input gate needed — it reads no input). Remove the `dock_haul_input` import.

### 2. `station` — broker any `Carrier`, drop the `rail` dep
- `systems.rs`: query `(With<Carrier>, With<Selected>)` instead of `(With<Locomotive>, …)`; rename
  the alias `SelectedLoco` → `SelectedCarrier`. Import `logistics::cargo_handling::components::Carrier`,
  drop `rail::components::Locomotive`. Logic (nearest station containing the mover → `CargoHaul` per
  `Storage` in radius) is otherwise unchanged.
- `bundle.rs`: `cargo_train` also inserts `Carrier` alongside `Inventory` + `Capacity`.
- `Cargo.toml`: remove `rail` (no longer referenced). Deps become bevy/range/logistics/selection.
- `lib.rs` + `components.rs`: fix the stale docs (#1 from the review) — the catchment is a
  `range::RangeRing`, **not** `DockZone`; the mover is "a `Carrier` (the player or a train)," not
  "a `Locomotive` with an `Inventory`."

### 3. `level` — plural authored objects, no demo, no routing
- `objects/manifest.rs`: `RawLevelSpec` → `storages: Vec<RawStorageSpec>`, `rails: Vec<RawRailSpec>`,
  add `stations: Vec<RawStationSpec>`. Drop `dock_radius` from `RawStorageSpec`. Add
  `RawStationSpec { pos: (f32,f32), facing: f32, radius: f32 }`.
- `objects/spec.rs`: mirror into `LevelSpec` (`storages`/`rails`/`stations` vecs), add `StationSpec`,
  drop `dock_radius` from `StorageSpec`, update the `From<Raw…>` impls (map the vecs).
- `objects/storage.rs`: remove `DockZone` (component, import, doc line).
- New `objects/station.rs`: thin constructor delegating to `station::bundle::station(pos, facing,
  radius)` (keeps `level`'s "objects build bundles" pattern; mirrors `objects/storage.rs`).
- `spawn.rs`: replace the single `storage`/`spawn_rail_line` calls with loops — one `storage` per
  `level.storages`, one `spawn_rail_line` + `cargo_train` insert per `level.rails`, one station per
  `level.stations`. `carrier_player` stays singular.
- Delete `station_demo.rs`; remove `pub mod station_demo` from `lib.rs`.
- `Cargo.toml`: remove `routing` (only `station_demo` used it; `routing` stays alive via `world`).
  Keep `station`, `world`, `rail`.

### 4. `app`
- `main.rs`: remove the `StationDemoPlugin` import and its entry in the plugin tuple. Keep
  `StationPlugin`. (`routing` dep was already dropped in the current changeset.)

### 5. `assets/level.ron` — rewrite to plural + a working station scene
Author (starter values; `level.ron` is hot-editable, so retuning needs no recompile):
- `storages: [ A @ (-600,-220) seeded with the current stock; B @ (800,380) empty ]`
  (drop the `dock_radius` field).
- `rails: [ the existing polyline, start: 0.0, heading: Forward ]` — its loco becomes a cargo train.
- `stations: [ A @ (-600,-330) facing 0.0 radius 200; B @ (800,290) facing 1.05 radius 200 ]`.
  Each 200-radius ring covers both its warehouse and the rail end it sits at, and the train at
  `s=0`/`s=length` lands inside it. Result: drive the train A→B, press `L` at A to load from
  warehouse A, `U` at B to unload into warehouse B — no routing involved.

### 6. Housekeeping
- Resolve the duplicate `p33`: rename `plans/p33_hud_content.md` → `plans/p35_hud_content.md`
  (keep `p33_rail_stations.md`); save this plan as `plans/p34_station_loading_cleanup.md`.
- Git ghost: `tests/multi_track.rs` is staged-added but worktree-deleted (its body now lives inline
  in `rail/src/systems.rs`). Manual `git` step for the user — I don't run VCS commands.

## Tests impacted (flagging per the no-silent-test-edit rule)

These break on the API/schema change and need updating in lockstep — I'll pause before touching each:
- `level/objects/storage.rs` test: drop the `DockZone` assertion and the `dock_radius` spec field.
- `level/objects/spec.rs` `parses_ron_into_in_memory_spec`: update to plural `storages`/`rails` +
  a `stations` entry; drop `dock_radius`.
- `station/tests/haul.rs`: spawn the mover with `Carrier` (or via `cargo_train`) instead of
  `Locomotive`; the three cases (in-range load, out-of-range silent, storage-out-of-radius) stay.
- Unaffected: `logistics` tests (none touch `DockZone`/`dock_haul_input`), `player.rs` test
  (`Carrier` stays), `rail/src/systems.rs` inline multi-track test.

## Verification
- `./bin/housekeeping.sh` clean (clippy + fmt, no warnings).
- `cargo test -p logistics -p station -p level -p rail` green.
- `cargo run -p pathfinding`: select the train, drive it (`D`) from station A to station B (`T`
  flips heading); at A press `L` (warehouse A stock flows into the train), at B press `U` (train
  empties into warehouse B). Select the player, stand in a station's ring, confirm `L`/`U` also
  brokers player↔warehouse. Confirm warehouses render no range disc; stations render a platform +
  faint ring.
