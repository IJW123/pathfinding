# Zoom + Metric Scale Bar

## Context

Today the world is measured in raw "world units" that coincide 1:1 with screen
pixels — but only because the `Camera2d` uses Bevy's default orthographic
projection (`ScalingMode::WindowSize`, `scale = 1.0`). That 1:1 is a default, not
a fact, and it wrongly conflates three layers:

1. **World space** — abstract simulation coordinates. We *define* them: **1 world
   unit = 1 metre.** Pixels never enter the simulation. (So a marching square,
   `ELEVATION_CELL = 10`, is 10 m; the full 4000-unit map is 4 km.)
2. **The projection** — the *only* bridge from world-metres to screen pixels.
   Zoom lives here.
3. **Screen pixels** — pure render output.

We're switching the projection to **`ScalingMode::FixedVertical`** so a fixed number
of *metres* is always visible regardless of window size/resolution — fully
decoupling units from pixels. Zoom then scales the visible metres. With units
defined as metres, no conversion constant is needed; "metres per pixel" is owned by
the camera and derived live for the scale bar.

The existing HUD is a **`Text2d` in world space** repositioned to `cam_pos +
window_half`, with a world-unit font size. That mixes pixels and world units and
breaks the moment we zoom, so it moves to screen-space Bevy UI. Required, not optional.

## Decisions (confirmed)
- 1 world unit = 1 metre (definitional; no conversion constant).
- Projection: `ScalingMode::FixedVertical` (resolution-independent), zoom via `scale`.
- Zoom input: hold **R** = zoom in, **F** = zoom out; centered; smooth/exponential.
- HUD fully metric: x/y as km/m, elevation z in m, speed in m/s.
- Scale bar picks a "nice" 1/2/5×10ⁿ-metre distance fitting a target pixel width,
  relabeled every frame from the camera's live visible area.

## Changes

### 1. `crates/camera_main` — projection + zoom
- `constants.rs`: add
  - `DEFAULT_VIEW_HEIGHT_M` — metres visible vertically at zoom 1 (e.g. `1000.0`).
  - `ZOOM_SPEED` (exponential rate), `ZOOM_MIN`, `ZOOM_MAX` — clamp on `scale`
    (e.g. `0.1` = ~100 m visible, `5.0` = ~5 km, covering the 4 km map). Tunable.
- `systems.rs`:
  - `spawn_camera`: spawn `(Camera2d, Projection::Orthographic(OrthographicProjection {
    scaling_mode: ScalingMode::FixedVertical { viewport_height: DEFAULT_VIEW_HEIGHT_M },
    ..OrthographicProjection::default_2d() }))`.
  - `zoom_camera(time, keyboard, Single<&mut Projection, With<Camera2d>>)`: match
    `Projection::Orthographic`, `scale *= ZOOM_SPEED.powf(dt)` (R) / inverse (F),
    `clamp(ZOOM_MIN, ZOOM_MAX)`.
- `plugin.rs`: add `zoom_camera` to `Update`.

### 2. `crates/hud` — screen-space UI + metric readout + scale bar
Replace world-space `Text2d` with a Bevy UI tree (flexbox; zoom/resolution
independent; drop the camera + window corner math entirely).

- `components.rs`: keep `HudReadout`; add `ScaleBarFill` (bar `Node`, dynamic width)
  and `ScaleBarLabel` markers.
- `constants.rs` (new): `TARGET_BAR_PX` (max bar width, e.g. `160.0`), bar height/colors.
- `format.rs` (new — keeps fns out of `mod.rs`/`lib.rs`):
  - `format_distance(metres: f32) -> String` → `"1.23 km"` if `|m| ≥ 1000` else `"123 m"` (sign-aware).
  - `nice_distance(raw_m: f32) -> f32` → round down to nearest 1/2/5×10ⁿ.
- `systems.rs`:
  - `spawn_hud`: root `Node`; top-right readout text node (`HudReadout`); bottom-left
    scale-bar group = `ScaleBarFill` bar + `ScaleBarLabel` text.
  - `update_hud_text`: read player `Transform`, `MeasuredVelocity`, `HeightField`;
    values are already metres — format x/y via `format_distance`, z and speed plain
    (`{:.1} m`, `{:.1} m/s`). Writes the UI `Text`. No camera/window queries.
  - `update_scale_bar(Single<&Projection, With<Camera2d>>, window, fill, label)`:
    from the orthographic `area` (visible world rect, in metres) and window px,
    `mpp = area.width() / window.width()`; `raw = TARGET_BAR_PX * mpp`;
    `nice = nice_distance(raw)`; set `ScaleBarFill` `Node` width to `nice / mpp` px;
    label = `format_distance(nice)`. (Uses computed `area` → works for any scaling mode.)
- `lib.rs`: add `pub mod format;` and `pub mod constants;`.
- `plugin.rs`: register `update_scale_bar` in `Update`.

## Notes / non-goals
- No `world` crate change — units are metres by definition.
- World-space content (contours, player, walls) scales correctly via the projection.
- Zoom-to-cursor and km/h speed are out of scope (trivial follow-ups).

## Verification
- `cargo run -p app`, then run housekeeping per CLAUDE.md; fix all warnings.
- Hold **R**/**F**: world scales smoothly about center, clamped both ends; resizing
  the window keeps the same world extent visible (FixedVertical working).
- Scale bar: while zooming, label steps through nice values (… 50 m, 100 m, 200 m,
  500 m, 1 km …) and bar stays near `TARGET_BAR_PX`; sanity-check length against a
  marching square (10 m) at high zoom.
- HUD: x/y in km/m, z in m, speed in m/s; pinned top-right / bottom-left at every
  zoom and window size, no drift, no font scaling.

## Follow-up: terrain not streaming into the expanded view when zoomed out

### Symptom
Zooming out reveals empty space at the edges — terrain chunks for the now-larger
visible area never load.

### Cause
Same pixel/world conflation this plan set out to kill, but in the chunk streamer.
`update_loaded_chunks` (`crates/world/src/elevation/chunk_lifecycle.rs`) computes
the visible extent as the **window size in logical pixels**:

```rust
let viewport = Vec2::new(window.width(), window.height());
let desired = desired_chunks(cam_pos, viewport);
```

`desired_chunks(cam_pos, viewport_size)` (`chunk_view.rs`) treats `viewport_size`
as a *world-unit* extent (`half = viewport_size * 0.5 + CHUNK_VIEW_MARGIN`, then
walks chunks across `cam_pos ± half`). Under the old `WindowSize`/scale-1.0
projection, 1 px == 1 world unit, so pixels happened to be the right number. With
`FixedVertical` + zoom, the visible world extent is `ortho.area` (world units =
`viewport_height · scale × aspect`), fully decoupled from window pixels. The
streamer keeps requesting a pixel-sized box, so anything past it stays unloaded
when zoomed out.

### Fix
Drive streaming from the camera's actual visible world rect, not window pixels —
the same `OrthographicProjection::area` the scale bar already uses (it's the view
rect in world units relative to the camera, auto-updated on resize/zoom).

In `update_loaded_chunks`:
- Add `projection: Single<&Projection, With<Camera2d>>`; drop the `window` query.
- `if let Projection::Orthographic(ortho) = projection.into_inner()` then
  `desired_chunks(cam_pos, ortho.area.size())`.

`Rect::size()` is the full visible width/height in world units, which is exactly
what `desired_chunks` expects as `viewport_size` — no change needed to
`chunk_view.rs` or the `CHUNK_VIEW_MARGIN` cushion.

Note this can load many chunks at max zoom-out (the 4 km map is ~12×12 chunks at
`ELEV_CHUNK_CELLS · ELEVATION_CELL = 320 m`/chunk — bounded by `MAP_HALF_EXTENT`
clamping, so the whole map at most). Acceptable for a fixed 4 km map; revisit only
if chunk build cost becomes visible.

### Verify
Zoom fully out: terrain fills the entire viewport with no empty margins; panning at
max zoom-out shows no unrendered edges. Zoom back in: distant chunks unload (watch
`LoadedChunks` count drop) so we don't retain the whole map needlessly.
