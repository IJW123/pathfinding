# p30 — Rail system with a single rail-bound locomotive

## Goal
A polyline rail track plus one locomotive constrained to it. The loco's only degree of freedom is
arc-length along the track, so it **physically cannot leave the rail**. Control:

- **Selectable** like the carrier. Click to select.
- While `Selected`: **Up** drives it along the track in its current heading.
- **T** flips heading 180° (the "turning button"); the same drive key then sends it the other way.
- Sprite always faces its travel direction.
- **Clamp at both ends** — dead stop, like buffer stops (no looping).

## Decisions (locked with user)
- Control: selectable, drive-along-rail. | Ends: clamp/dead-stop. | Turn = 180° heading flip.

## Architecture fit (verified against codebase)
- Movement is 1-D (arc-length `s`); a projector derives world `Transform` from `s` each tick →
  the rail constraint is structural, not enforced after the fact.
- The loco needs a `Collider` for click-picking (`select_on_click` hit-tests `Collider`), but is
  **not `Solid`**. `sync_physics_world` inserts every `Collider` into rapier, but the solver only
  pushes pairs where `both_solid` (pinned by `non_solid_overlap_reported_not_resolved`). So the loco
  is detected but never shoved off the rail; the projector stays authoritative.
- Rail layout is **data in `level.ron`** (consistent with the "level is data" pattern), loaded into
  `LevelSpec`, spawned by `spawn_level`.
- Rendering lives only in `render` (one-way dep: render → rail).

## New crate: `rail` (world logic)
`lib.rs` = `pub mod` lines only (no items in mod/lib per house rules).

- `smooth.rs` — `smooth_track(authored: &[Vec2], samples_per_span: usize) -> Vec<Vec2>`:
  **centripetal Catmull-Rom** through the authored waypoints, flattened to a dense polyline so
  corners round off. Interpolates the waypoints (track passes *through* them); first/last points are
  duplicated as phantom neighbors so the endpoints stay exactly on the authored ends (dead-stop lands
  where authored). Pure `Vec<Vec2> → Vec<Vec2>`, isolated and swappable — the rest of the pipeline
  never sees the smoothing. Resolution from `CURVE_SAMPLES_PER_SPAN` (constants). Benefits **every**
  producer: a future grid pathfinder's staircase path rounds off here for free (path *simplification*
  stays the producer's job).
- `track.rs` — `RailTrack` (Component): the **smoothed** polyline `points: Vec<Vec2>` + precomputed
  cumulative arc-lengths. `RailTrack::new(authored)` runs `smooth_track` then accumulates lengths
  (`debug_assert!` ≥2 authored points and non-zero total length — a degenerate track makes `sample`
  divide by a zero-length segment and makes `s.clamp(0, 0)` pin the loco), `length()`,
  `sample(s) -> RailPose { position, angle }` (clamps `s`, finds segment, lerps, tangent → angle;
  with the dense curve the tangent now turns continuously through corners), `points()` accessor for
  the renderer. **This constructor is the durable seam**: RON is just today's producer; a
  user/AI pathfinder later feeds the same `Vec<Vec2>` without `rail` ever learning about RON.
- `components.rs` — `Locomotive` marker `#[require(Selectable, FreeMoveExempt)]`;
  `TrackPosition(pub f32)` (arc-length); `RailHeading { Forward, Backward }` with
  `sign() -> f32` and `flipped() -> Self`.
- `constants.rs` — `LOCO_SPEED`, loco collider half-extents, `LOCO_Z` (=1.0, alongside the player),
  `TRACK_Z` (≈0.2, above ground/walls, below obstacles) — the track entity's Transform Z, a
  *world-side* constant here, matching the `PLAYER_Z` precedent in `player/constants.rs`; the render
  crate owns colors only (LineStrip has no width), not Z-ordering. `DRIVE_KEY = ArrowUp`,
  `TURN_KEY = KeyT`.
- `systems.rs`
  - `turn_locomotive` (**Update**, not FixedUpdate): query loco `&mut RailHeading` filtered
    `(With<Locomotive>, With<Selected>)`. On `T` `just_pressed` → flip heading. Edge-detected input
    MUST live in `Update`; in `FixedUpdate` it can fire 0× or 2× in one frame at off-nominal
    framerates → dropped or double (self-cancelling) flips. Mirrors `select_on_click`, which already
    reads mouse `just_pressed` in `Update`.
  - `drive_locomotive` (FixedUpdate): query loco `(&mut TrackPosition, &RailHeading)` filtered
    `(With<Locomotive>, With<Selected>)` + `Single<&RailTrack>`. While `Up` is *held* (`pressed`, a
    level not an edge → safe under FixedUpdate's variable run-count) →
    `s = (s + LOCO_SPEED*dt*heading.sign()).clamp(0, len)`.
  - `project_locomotive` (FixedUpdate, after drive): `let pose = sample(s)`; set
    `transform.translation = pose.position.extend(LOCO_Z)` (preserve the loco's Z layer — `sample`
    only yields a `Vec2`) and `transform.rotation = Quat::from_rotation_z(pose.angle + offset)` with
    `offset` = `0` (Forward) / `PI` (Backward). Runs every tick (not gated on `Selected`) so the loco
    always sits on the rail.
- `bundle.rs` — `locomotive(track: &RailTrack, start_s, heading) -> impl Bundle`
  (Locomotive, Collider::obb(..), TrackPosition, heading, initial `Transform` from `sample` at
  `LOCO_Z`); `rail_track(track: RailTrack) -> impl Bundle` = `(track, Transform` at `TRACK_Z)` — the
  track entity needs a `Transform` so its mesh layers correctly and the renderer has a node to attach
  to.
- `plugin.rs` — `RailPlugin`: `add_systems(Update, turn_locomotive)` +
  `add_systems(FixedUpdate, (drive_locomotive, project_locomotive).chain())`. No ordering vs
  `CollisionSet` is required for correctness (the loco is non-solid, so collision never moves it, and
  it carries no `MeasuredVelocity`); `.before(CollisionSet)` on the FixedUpdate chain is optional
  polish so rapier sees the fresh pose the same tick.

## `selection` change (keeps the dep graph one-way)
`move_selected` (currently `Query<&mut Transform, (With<Selected>, Without<Static>)>`) would
otherwise free-move the selected loco in xy, fighting the projector. Add an **opt-out marker
`FreeMoveExempt`** to `selection::components` and extend the filter to
`(With<Selected>, Without<Static>, Without<FreeMoveExempt>)`. `rail` depends on `selection` and tags
the loco; `selection` never learns about `rail`. The naive alternative — filtering
`Without<TrackPosition>` — would force `selection → rail` (a cycle); a generic `selection`-owned
marker avoids it. Chosen over relying on projector-overwrite ordering (too implicit) — this is
explicit and modular.

## `render/rail` module
- `mesh.rs` — `track_line_mesh(points) -> Mesh`: a `PrimitiveTopology::LineStrip` over the smoothed
  polyline. Reuses the *technique* from `contour_lines_to_mesh` (cheap GPU line mesh), **not**
  marching squares — the rail is authored/generated, not an iso-contour of any field. Trade-off: 1px
  line that doesn't thicken on zoom; swap to stitched quads later if a chunky rail is wanted.
- `constants.rs` — `RAIL_COLOR`, `LOCO_COLOR`. (Z-ordering is **not** here: the track entity carries
  `TRACK_Z` from `rail/constants.rs` per the `PLAYER_Z` precedent, so render stays purely visual.)
- `plugin.rs` — `RailRenderPlugin`: `attach_track_mesh` (on `Added<RailTrack>`, build mesh from
  `points()`), `attach_locomotive_sprite` (on `Added<Locomotive>` w/ `&Collider`, colored sprite
  sized to `render_size()` — mirrors `attach_player_sprite`).
- Wire into `render/lib.rs` + `RenderPlugin`.

## `level` integration (scaffolding — RON is today's producer, not the final one)
- `manifest.rs` — `RawRailSpec { points: Vec<(f32,f32)>, start: f32, heading: RawHeading }`,
  `RawHeading { Forward, Backward }` + inherent `to_heading() -> RailHeading` (orphan rule, like
  `RawCommodity::to_commodity`). Add `rail: RawRailSpec` to `RawLevelSpec`.
- `spec.rs` — `RailSpec { points: Vec<Vec2>, start: f32, heading: RailHeading }`, add `rail` to
  `LevelSpec`, `From` mapping.
- `spawn.rs` — build `RailTrack::new(points)`, spawn `locomotive(&track, start, heading)` then
  `rail_track(track)`.
- `level.ron` — add a `rail:` block (a short L-shaped polyline + `start`/`heading`).
- `level/Cargo.toml` — add `rail`.

## Workspace wiring
- Root `Cargo.toml`: add `crates/rail` to members + `rail = { path = "crates/rail" }` to workspace deps.
- `app/main.rs`: add `RailPlugin` (RenderPlugin already nests RailRenderPlugin).

## Tests
- `rail/smooth.rs`: smoothed curve passes through the authored waypoints; endpoints land exactly on
  authored first/last; a right-angle corner yields a turn with no tangent discontinuity (sample
  either side of the corner → bounded angle delta).
- `rail/track.rs`: sample at `s=0` / `s=len` / midpoint; clamp past both ends; straight + L-shaped
  polylines; tangent-angle correctness. `RailHeading::flipped`/`sign`.
- `rail/bundle.rs`: loco has Locomotive, Collider, TrackPosition, heading, initial on-rail Transform.
- **⚠ Test edits requiring your OK (per test policy):**
  - `level/src/objects/spec.rs::parses_ron_into_in_memory_spec` — inline RON gains a `rail:` block +
    a couple of assertions (otherwise the new required field fails to parse).
  - `level/tests/spawn_level.rs::sample_level` — add a `rail` field to the literal `LevelSpec`
    (otherwise it won't compile); add a "1 locomotive" count assertion.

## Out of scope (note for future)
- **User/AI rail placement via a pathfinding algorithm** — the eventual real producer. It will feed
  `RailTrack::new(Vec<Vec2>)` directly (likely a separate `pathfinding` crate), making the RON path a
  throwaway front-end. Designing the constructor as the single seam now is what keeps that drop-in.
- No collision response for the loco (rail-constrained; it emits harmless contact events).
- Single track, single loco. Multi-track / branching / coupling cars deferred.
- No accel/decel or HUD readout for the loco.

## Housekeeping
Run `./bin/housekeeping.sh` after implementation; fix all warnings.

<!-- auto-reviewed -->
