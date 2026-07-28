# p36: Derived stations at rail terminals + platform slim-down

## Context

Evaluation of the p33/p34 station work surfaced three issues:

1. **Big: stations don't lock onto rails.** `StationSpec.pos`/`facing` are hand-tuned RON values: station A sits 70 units off the rail start; station B's `facing: 1.05` approximates the tangent. Editing either side silently breaks the pairing, and track smoothing means authored coords never exactly hit the final curve anyway.
2. **Minor: platform bar too wide.** `PLATFORM_WIDTH` 28.0 → 14.0.
3. **Question (answered, no code change): warehouse alignment.** None exists — interaction is pure radial distance via `RangeRing`; `facing` is cosmetic. (Known soft spot, out of scope: `Storage.half_extent` ignored in the radius check.)

**Direction:** rails will later be built at runtime (manually and by an AI) to connect *loading/unloading places* — warehouses, for now. A rail may also end somewhere with no such feature nearby. So neither side snaps to the other; instead **stations are derived**: a rail terminal that lands near a warehouse gets a station born *on* the terminal — locked by construction, exact tangent, no projection math. A terminal near nothing gets no station. AI-built rails later get stations the same way. Radius is a constant for now; no station→track entity link yet.

## Approach

Delete authored stations. At spawn, for each rail line, sample both terminal poses (`track.sample(0)` / `track.sample(track.length())`); if any warehouse lies within `STATION_RADIUS` of a terminal, spawn a station there with that pose's tangent as facing. Current level data already satisfies this: both rail terminals are 180 units from their warehouses (< 200).

## Changes

### 1. `crates/station/src/constants.rs`
`STATION_RADIUS: f32 = 200.0` **already exists** (line 9) — nothing in code references it yet; the RON just happened to author the same value. No addition; reword its doc from "Default catchment radius" to "Catchment radius of a derived station" since this change makes it load-bearing.

### 2. `crates/level/src/objects/station.rs` — derivation, not authoring
Replace the spec-mapping constructor with:
```rust
/// Stations a finished track earns: one at each terminal that has a loading/
/// unloading place (warehouse) within STATION_RADIUS. Pose comes from the
/// track itself, so the platform is on the rail by construction.
pub fn terminal_stations(track: &RailTrack, storage_positions: &[Vec2]) -> Vec<impl Bundle>
```
For `s in [0.0, track.length()]`: `pose = track.sample(s)`; if any storage position within `STATION_RADIUS` of `pose.position` → `station_bundle(pose.position, pose.angle, STATION_RADIUS)`. `station::bundle::station` unchanged. Module doc rewritten (it currently narrates the spec-mapping role).

Why `level`, not `station`: `station` has no `rail` dependency (its Cargo.toml: bevy/range/logistics/selection — the "rail bridge" is `cargo_train` inserted on the loco *in level*). Taking `&RailTrack` in `station` would add that coupling for one function; `level` already depends on both, and the fn stays `pub` so a future runtime rail-building system can call the same seam.

**Degenerate-track guard:** `RailTrack::sample` pins a `< 2`-point track to a single pose (its release safety net), and a zero-length track makes `[0.0, length]` sample the same spot twice — either way two stacked stations. Guard: only take the far terminal when `track.length() > 0.0`. Matters because this fn is the seam a future pathfinder feeds.

### 3. `crates/level/src/spawn.rs`
Collect `storage_positions` from `level.storages`. In the rail loop, before `spawn_rail_line` consumes the track, spawn `terminal_stations(&track, &storage_positions)`. Delete the authored-stations loop.

### 4. `crates/level/src/objects/manifest.rs` + `spec.rs` — remove authored stations
Delete `RawStationSpec` / `StationSpec`, the `stations` fields on both level specs, and the `From` impl.

**Test edits (flagged per test policy):**
- `spec.rs::parses_ron_into_in_memory_spec`: remove the `stations:` block from the inline RON text and the two station assertions (pos/radius round-trip) — the type they test is deleted. Nothing else touched.
- `tests/spawn_level.rs`: remove the `stations: vec![]` field from the spec literal (mechanical; field is gone).

### 5. `assets/level.ron`
Delete the `stations:` block. Update the storages comment: a warehouse within `STATION_RADIUS` of a rail terminal gets a station derived at that terminal; rails ending elsewhere get none.

### 6. `crates/render/src/station/constants.rs`
`PLATFORM_WIDTH: 28.0` → `14.0`.

## Not doing (deliberately)
- No runtime `Added<RailTrack>` derivation system — spawn-time only until runtime rail-building exists; `terminal_stations` is the reusable seam.
- No station→track entity link (no consumer yet).
- No mid-track stations (terminals only — matches "rails connect load/unload places").
- No dedup if two tracks terminate at the same warehouse (two platforms is acceptable). Same acceptance for a short track whose *both* terminals reach one warehouse — two stations, fine.
- No `half_extent`-aware radius check (separate issue).

## New tests (additions only)
`level`: `terminal_stations` — terminal with a warehouse in range yields a station at the exact terminal pose; terminal with none yields nothing; both-ends case yields two; degenerate (<2-point) track yields at most one, never two stacked. Note: `impl Bundle` is opaque — assert by spawning the bundles into a minimal `App` and querying `Transform`/`RangeRing` (same headless pattern as `tests/spawn_level.rs`).

`tests/spawn_level.rs` (addition, existing assertions untouched): the sample level's storage is ~570+ units from both rail terminals, so assert **zero** `Station` entities spawn — locks in "terminal near nothing gets no station" at the integration boundary.

## Verification
- `cargo test -p level` (`rail`/`station` untouched; housekeeping covers the workspace).
- `cargo run -p pathfinding` — stations appear exactly at both track ends, platform on the rail and tangent-aligned (curved end at station B is the visual tell); bar half as wide; L/U haul works at both ends for train and player.
- `./bin/housekeeping.sh` clean.

<!-- auto-reviewed -->
