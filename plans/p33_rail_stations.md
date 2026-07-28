# Plan: Rail stations (broker model)

## Context

The bespoke `warehouse_link` demo (hardcoded "spawn warehouse B, route a rail A→B") was a
dead-end: it conflated building placement with rail building and lived in the binary. The
sustainable concept is a **station** — a rail-side node with a load/offload radius. Rail lines
connect stations; a train stopped at a station exchanges cargo with the warehouses in that
station's radius. This generalizes to in-game building later (a station is just an entity with a
radius; the transfer is a message), and it replaces the warehouse_link scaffolding.

Decisions (from discussion): **broker** model — the station holds no stock, it brokers between the
train and `Storage` within its radius; **manual** trigger — a key on the selected train; **scope** —
stations replace the warehouse demo (train carries cargo; a routed rail connects two stations; the
train transfers with warehouses in each station's radius).

Almost everything is reuse:
- `DockZone { radius }` (`logistics/src/cargo_handling/components.rs:10`) — the radius concept.
- `CargoHaul { source, dest }` message + `apply_cargo_hauls` (`logistics/src/cargo_handling/{message,systems}.rs`)
  — the transfer engine, already registered by `LogisticsPlugin`; moves all commodities respecting
  `Capacity` (per-commodity `min(available, grantable)`, drops `source == dest`/missing endpoints).
  Used unchanged.
- **`dock_haul_input`** (`logistics/src/cargo_handling/systems.rs:36`) — the existing template:
  `Carrier` near the nearest in-range `Storage` `DockZone`, Digit9/Digit0, picks one storage via
  `here.distance(tf.xy()) <= radius` then `min_by(dist)`, writes one `CargoHaul`. `station_haul_input`
  mirrors its conventions (2D `.xy()` distance, `just_pressed` edge, `min_by` nearest) but with the
  broker topology (loco → nearest *station* → *every* `Storage` in that station's radius).
  **The two coexist** (different keys, different trigger entity). That's the deliberate scope of this
  pass; folding the direct carrier-dock path into the station model is a later cleanup, not now.
- `Inventory` / `Capacity` (`logistics/src/components.rs:19,115`), `Storage` marker, `storage_building`.
- `routing::find_path` + `PathProfile`, `rail::spawn::spawn_rail_line`, `selection::Selected`.

## New crate `crates/station` (the feature; bridges rail + logistics)

Deps: `bevy`, `rail`, `logistics`, `selection`. (`station` is a leaf above rail/logistics — no cycle;
rail and logistics stay independent and logistics-free-of-rail.)

- `lib.rs` — `pub mod` declarations only.
- `components.rs` —
  ```rust
  /// A rail-side transfer node. Holds no goods; brokers cargo between a train inside its `DockZone`
  /// radius and the `Storage` buildings in that same radius. Reuses logistics' `DockZone` so a
  /// station's catchment and a warehouse's dock share one radius concept.
  #[derive(Component)]
  pub struct Station;
  ```
- `bundle.rs` —
  - `station(transform: Transform, radius: f32) -> impl Bundle` = `(Station, transform, DockZone { radius })`.
  - `cargo_train(capacity: Capacity) -> impl Bundle` = `(Inventory::default(), capacity)` — inserted
    onto a locomotive entity to make it haul cargo (kept here, not in `rail`, so rail stays
    logistics-free). **No `Carrier`** — that marker is what `dock_haul_input` keys on; adding it would
    make the loco double-handled by both haul systems.
- `systems.rs` — `station_haul_input`: on `LOAD_KEY`/`UNLOAD_KEY` (`just_pressed`), take the
  `Selected` `Locomotive`; find the nearest `Station` whose radius contains it (2D `.xy()` distance,
  `min_by` — same as `dock_haul_input`); for each `Storage` whose center is within that station's
  radius, write a `CargoHaul` (load: `storage → loco`, unload: `loco → storage`). `apply_cargo_hauls`
  does the rest. No "stopped" check this pass — the player stops then presses. Define a local
  `type SelectedLoco = (With<Locomotive>, With<Selected>)` (rail's alias is private). **No-op, don't
  panic, on 0 or >1 selected locos** — iterate the filtered query, bail if not exactly one (no
  `.single()` unwrap).
- `plugin.rs` — `StationPlugin`: adds `station_haul_input` to `Update`
  (`run_if(resource_exists::<ButtonInput<KeyCode>>)`). Does **not** register `CargoHaul` — relies on
  `LogisticsPlugin` having done `add_message::<CargoHaul>` + `apply_cargo_hauls`, so `StationPlugin`
  must be added alongside `LogisticsPlugin` (it is, in `main.rs`). Cross-plugin ordering vs
  `apply_cargo_hauls` isn't required — Bevy messages persist a frame, so a same-frame miss just
  applies next frame (harmless for cargo). Note it rather than reach for a shared `SystemSet`.
- `constants.rs` — `LOAD_KEY`/`UNLOAD_KEY` (e.g. `KeyL`/`KeyU`), `TRAIN_MAX_WEIGHT`,
  `TRAIN_MAX_VOLUME`, default station `RADIUS`.

## `crates/rail` — return the loco entity + relocate the multi-track test

- `spawn_rail_line` currently returns only the track entity, but the demo must augment the loco with
  cargo. Change it to return both:
  ```rust
  pub struct RailLine { pub track: Entity, pub loco: Entity }
  ```
  The current impl does `commands.spawn(locomotive(...))` and **drops the loco id** — capture it via
  `.id()` to populate `RailLine.loco`. Update the two existing callers (`level/src/spawn.rs`, the
  moved demo) — they currently discard the return, so this is additive.
- **Cleanup 1 (placement fix):** move the multi-track regression test from
  `crates/rail/tests/multi_track.rs` into a `#[cfg(test)] mod tests` in `systems.rs`, then delete the
  now-empty `tests/` file. It tests `project_locomotive` (which lives in `systems.rs`), and every
  other rail test is colocated in `src/` — the lone `tests/` file was inconsistent with the crate.
  Same body/`run_system_once` approach; `super::*` replaces the `rail::` import paths.

## `crates/level` — the demo bootstrap (Cleanup 2: out of the binary)

The `app/src/warehouse_link.rs` bootstrap moves to `crates/level/src/station_demo.rs` as
`StationDemoPlugin`, so the binary returns to pure plugin composition (`main.rs` only — no non-`main`
source). It's world-population content, which is `level`'s job. `level` gains `routing` + `station`
deps (both leaves above it — no cycle; `level` already has `world`/`rail`). The system, at `Startup`
with `LevelSpec` + `HeightField`:
- Station A at the authored warehouse's position (`level.storage.pos`, `-250,200`) with `RADIUS`, so
  the existing warehouse sits in its catchment.
- A second warehouse (`storage(&StorageSpec{..})`) + Station B at `~(1100,-500)`.
- `find_path(station_a, station_b, rail_profile)` → `RailTrack::new(path)` → `spawn_rail_line(...)`
  → insert `cargo_train(Capacity { max_weight: Some(TRAIN_MAX_WEIGHT), max_volume: Some(TRAIN_MAX_VOLUME) })`
  onto the returned `loco`. (The `RailTrack::new` step is existing warehouse_link logic, kept as-is.)
- The two placement positions live in a module-local `constants` per policy; `RADIUS`, `TRAIN_MAX_*`
  come from `station::constants`.

`StationDemoPlugin` is a **separate** plugin, **not** part of `LevelPlugin` — so the `spawn_level`
integration test (which adds only `LevelPlugin`) never runs it and stays `HeightField`-free, and
`spawn_level` itself is untouched. `level/src/lib.rs` gets `pub mod station_demo;`.

`app/main.rs` adds `station::StationPlugin` (the feature) + `level::StationDemoPlugin` (content), and
**deletes its `warehouse_link` module entirely**. `app/Cargo.toml`: add `station`; drop `routing`
(only the moved demo used it) and the `ron` dev-dep (the route test moves to `level`, which already
depends on `ron`).

> Note: this is still hardcoded demo scaffolding — just no longer in the binary. Promoting
> stations/warehouses/rail-lines to `level.ron` data is the separate "graduate it" task deferred
> earlier.

## Tests

- **`crates/station/tests/haul.rs`** (new) — `world.init_resource::<Messages<CargoHaul>>()` (or build a
  minimal `App` with `add_message::<CargoHaul>()` — `MessageWriter` needs the resource to exist or
  `run_system_once` errors), spawn a `Selected` loco (with `Inventory`+`Capacity`) inside a station's
  radius, a stocked `Storage` in range, insert `ButtonInput<KeyCode>` with `LOAD_KEY` pressed via
  `.press(...)` (sets `just_pressed`), run `station_haul_input` via `run_system_once`, and assert a
  `CargoHaul { source: storage, dest: loco }` was emitted (drain `Messages<CargoHaul>`). A second
  case: loco outside every station radius ⇒ no message. (Dev-dep on `logistics` for `CargoHaul`/
  `Storage`/`Inventory`.)
- **route test** — moves with the bootstrap into `crates/level/src/station_demo.rs`, unchanged in
  substance (route between the two station positions exists over real terrain). Reads
  `assets/level.ron` via `level`'s `CARGO_MANIFEST_DIR` (`../../assets/...`).
- **`crates/rail`** — the multi-track test is now a colocated `systems.rs` unit test (Cleanup 1);
  other rail tests unaffected; `spawn_rail_line` return change compiles against callers (bundle tests
  don't use it).

## Verification

- `cargo test -p station` — haul-emit + out-of-range tests pass.
- `cargo test -p rail` — relocated multi-track test green in its new `systems.rs` home.
- `cargo test -p level` — `spawn_level` still spawns exactly 1 loco (unaffected by the demo plugin)
  + the moved route test passes.
- `cargo test -p pathfinding@0.1.0` — app still builds/tests (`@0.1.0` disambiguates the local app
  crate from the external `pathfinding` dep — name collision).
- `./bin/housekeeping.sh` — clippy + fmt clean.
- `cargo run --bin pathfinding`: select the train, drive (↑) to warehouse A, press `LOAD_KEY` — the
  debug inventory log (`logistics::debug::log_inventory_changes`) shows goods moving warehouse→train;
  drive to warehouse B, press `UNLOAD_KEY` — goods move train→warehouse. (HUD cargo readout is out of
  scope; the loco has no `MeasuredVelocity`, so the existing HUD already shows nothing for it — a
  separate follow-up.)

<!-- auto-reviewed -->
