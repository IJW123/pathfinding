# Plan: Per-locomotive track association (multi-track rail)

## Context

Adding a second `RailTrack` (the routed warehouse rail) silently broke *all* locomotive
movement. Root cause: `drive_locomotive` and `project_locomotive` take `Single<&RailTrack>`
(`crates/rail/src/systems.rs:33,49`), and Bevy's `Single` **skips the whole system** when the
query doesn't resolve to exactly one entity. Two tracks ⇒ both systems no-op ⇒ nothing moves.
The rail crate has a baked-in "one global track" assumption.

Fix it sustainably: each `Locomotive` carries an explicit `OnTrack(Entity)` pointing at the
track it rides; the drive/project systems look the `RailTrack` up by that entity. Any number of
independent tracks + locos then coexist. Decision: the hand-authored placeholder rail and the
routed warehouse rail **both stay**, as two separately-drivable tracks.

## Changes

### `crates/rail` — the real fix

- **`src/components.rs`** — new component:
  ```rust
  /// The track entity this locomotive rides. Drive/project resolve the `RailTrack` by this id,
  /// so tracks and locos are many-to-… not a global singleton.
  #[derive(Component, Debug, Clone, Copy)]
  pub struct OnTrack(pub Entity);
  ```
  A plain `Entity` newtype, not a Bevy 0.18 `#[relationship]`: there's no reverse-lookup need
  (we only ever go loco → track) and no automatic despawn cascade wanted, so the relationship
  machinery would be dead weight. KISS.
- **`src/components.rs`** — also fix the now-false `Locomotive` doc comment: it opens
  "The single rail-bound locomotive." Multiple coexisting locos are the whole point of this
  change; reword to drop the singleton claim. (Doc-only, no behavior.)
- **`src/bundle.rs`** — `locomotive` gains a `track_entity: Entity` param and inserts
  `OnTrack(track_entity)` alongside the existing components. Signature becomes
  `locomotive(track: &RailTrack, track_entity: Entity, start_s: f32, heading: RailHeading)`.
- **`src/spawn.rs`** (new, `pub mod spawn;` in `lib.rs`) — centralize the spawn ordering so no
  caller re-derives it:
  ```rust
  /// Spawn a rail line: the track entity plus a loco riding it. Reserve the track entity id up
  /// front so the loco can carry `OnTrack(id)` while `track` is still borrowable — `RailTrack`
  /// is not `Clone`, and `locomotive` borrows `&track` to seed its start pose whereas
  /// `rail_track(track)` consumes it, so the consuming insert must come last. Returns the track
  /// entity. Both call sites currently discard it; it's returned for callers that later need to
  /// wire more onto the track.
  pub fn spawn_rail_line(commands: &mut Commands, track: RailTrack, start_s: f32,
                         heading: RailHeading) -> Entity {
      let track_entity = commands.spawn_empty().id();
      commands.spawn(locomotive(&track, track_entity, start_s, heading));
      commands.entity(track_entity).insert(rail_track(track));
      track_entity
  }
  ```
  This is genuine DRY, not premature abstraction: the borrow-vs-move ordering above is subtle
  enough that two callers open-coding it would be the bug factory. `OnTrack` is inserted by the
  `locomotive` bundle, not here — the helper only owns spawn ordering.
- **`src/systems.rs`** — drop `Single<&RailTrack>`; add `tracks: Query<&RailTrack>` and `&OnTrack`
  to the loco queries, resolving the track per loco (`let Ok(track) = tracks.get(on_track.0) else
  { continue };`). In `drive_locomotive` the `let length = track.length()` moves *inside* the loop —
  it was hoisted above the loop under the single-track assumption, which no longer holds.
  `project_locomotive` samples each loco's own track.
  - **Borrow conflict — no, but for the right reason.** The track entity (via `rail_track`) also
    carries a `Transform`, and the loco query takes `&mut Transform`. Bevy resolves access at the
    component level, not the entity level: `tracks: Query<&RailTrack>` never touches `Transform`,
    so the two queries' component sets are disjoint and the system builds. The `With<Locomotive>`
    filter is incidental to that, not what saves it.
  - **Dangling `OnTrack` is intended, not a silent bug.** If a track is ever despawned,
    `tracks.get(on_track.0)` returns `Err` and the loco is skipped (`continue`) — it freezes at its
    last pose rather than panicking. No track is despawned today, so this is latent; call it out so
    the `continue` reads as a deliberate "orphaned loco idles," not an overlooked case.
  - If clippy's `type_complexity` trips on `project_locomotive`'s 4-tuple query, alias it the way
    `SelectedLoco` already aliases the driver filter.

### Call sites — use the helper

- **`crates/level/src/spawn.rs`** — revert the earlier placeholder removal; respawn the authored
  rail through the helper: `spawn_rail_line(&mut commands, RailTrack::new(level.rail.points.clone()),
  level.rail.start, level.rail.heading);` (restores `use rail::spawn::spawn_rail_line;` +
  `use rail::track::RailTrack;`). The authored track is back ⇒ the `level` integration test is
  untouched and still valid.
- **`crates/app/src/warehouse_link.rs`** — replace the manual track+loco spawn in the `Some`
  arm with `spawn_rail_line(&mut commands, RailTrack::new(points), 0.0, RailHeading::Forward);`.
  Drop the now-unused `locomotive`/`rail_track` imports.

## Tests

- **`crates/rail/src/bundle.rs`** (edit — mechanical, plan-approved): the existing
  `locomotive_has_full_composition_and_sits_on_rail` call gains a track-entity arg
  (`Entity::PLACEHOLDER`) and asserts `OnTrack` is present and equals it. Intent/coverage
  preserved, just the new association added.
- **`crates/rail/tests/multi_track.rs`** (new) — the regression guard for the actual bug: spawn
  **two** `RailTrack` entities and a loco on each (`OnTrack` set accordingly), then run
  `project_locomotive` once via `world.run_system_once` (bevy 0.18) — it has only `Query` params
  and no run conditions, so it needs no schedule or resources — and assert each loco's `Transform`
  lands on *its own* track. Use tracks with clearly different geometry so a mixed-up lookup can't
  accidentally pass. Proves two tracks coexist — the exact thing `Single` forbade.
- **`crates/level/tests/spawn_level.rs`** — unchanged; still asserts 1 locomotive (authored rail
  restored).
- **`crates/app/src/warehouse_link.rs`** route test — unchanged.

## Verification

- `cargo test -p rail` — updated bundle test + new `multi_track` test pass.
- `cargo test -p level` — `spawn_level` still spawns exactly 1 loco.
- `cargo test -p pathfinding@0.1.0` — warehouse route test still passes (`-p pathfinding` is
  ambiguous; the external `pathfinding` crate shares the name).
- `./bin/housekeeping.sh` — clippy + fmt clean.
- `cargo run --bin pathfinding` — click a locomotive to select it, hold ↑: it drives along its
  own track; `T` flips heading. Both the authored rail (SW) and the routed warehouse rail are
  independently drivable. Note: only the *selected* loco drives/turns (`SelectedLoco` filter);
  `project_locomotive` keeps every loco glued to its rail regardless of selection — so the
  unselected one should sit still on-rail, not drift.
<!-- auto-reviewed -->
