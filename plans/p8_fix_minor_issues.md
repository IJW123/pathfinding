# Plan: Minor fixes from the terrain-scale review

## Context

Cleanup pass on smaller issues found reviewing the zoom + scale-bar work (`p6`). None are
architectural (that's `p7`); these are latent panics, an Inf path, a one-frame load spike,
and tooling/doc drift. Independent, can land in any order — but **after `p7`**, since #1
edits the same `update_loaded_chunks` that `p7` reshapes.

## 1. Per-frame chunk-load budget (load spike at max zoom-out)

**Problem:** at `ZOOM_MAX = 5.0` ~5 km is visible and the map is 4 km, so the entire
~13×13 chunk grid becomes "desired" in a single frame → ~150+ chunk spawns + contour
extractions in one tick → frame stall. The cache makes it one-time, but it's a visible
hitch, and nothing bounds loads per frame. (Ties to the known elevation-lifecycle
fragilities.)

**Fix:** cap new spawns per frame in `update_loaded_chunks`
(`crates/world/src/elevation/chunk_lifecycle.rs`). Add
`MAX_CHUNK_LOADS_PER_FRAME` to `crates/world/src/elevation/constants.rs`; load at most that
many missing chunks per tick (`desired` is recomputed every frame, so the remainder is
picked up over subsequent frames). Unloads can stay immediate. Keep it simple — a counter
in the spawn loop, `break` when the budget is spent.

**Note:** silent truncation reads as "loaded everything" — fine here because the next frame
continues, but don't add a cap elsewhere without the recompute-every-frame guarantee.

## 2. `Single` → `Option<Single>` where the entity can vanish

**Problem:** `update_hud_text` uses `Single<… With<Player>>`; `Single` panics on zero-or-many
matches. The moment the player despawns (death/respawn) the HUD system crashes.

**Fix:** `crates/hud/src/systems.rs` → `update_hud_text`: take
`player: Option<Single<(&Transform, &MeasuredVelocity), With<Player>>>` and early-return
(implicit) when `None`. Camera `Single`s (`zoom_camera`, `update_scale_bar`,
`sync_stream_focus`) are lower risk — the camera is spawned once at startup and never
despawned — so leave them unless/until a second camera or teardown appears. Flagging, not
changing.

## 3. Scale-bar divide-by-zero / first-frame Inf

**Problem:** `update_scale_bar` computes `mpp = ortho.area.width() / window.width()` then
fill width `= nice / mpp`. Bevy computes `ortho.area` in `PostUpdate`, so on the first
`Update` `area` can be zero → `mpp = 0` → width `= Inf` → one frame of a garbage-width node.

**Fix:** `crates/hud/src/systems.rs` → `update_scale_bar`: guard before writing —
`if window.width() > 0.0 && mpp.is_finite() && mpp > 0.0` (else leave the bar as-is this
frame). Cosmetic, but removes a real NaN/Inf path.

## 4. Order projection readers after the zoom writer (low priority)

**Problem:** `zoom_camera` writes `Projection`; `update_scale_bar` (and, post-`p7`,
`sync_stream_focus`) read it with no ordering → readers may use last-frame scale (≤1-frame
lag after a zoom). Imperceptible, nondeterministic.

**Fix (optional):** order the readers `.after(zoom_camera)` in the relevant plugins
(`camera_main`/`hud`/`app`). Skip if not worth the wiring; documented so it's a known
choice, not an oversight.

## 5. Housekeeping script + CLAUDE.md drift

`bin/housekeeping.sh`:
- **Shebang `#!/usr/bin/env zsh`** but the dev shell is bash → fails if zsh absent. Change to
  `#!/usr/bin/env bash` (script is POSIX-ish; no zsh-only syntax).
- **All lints are `-W`** (warn), no `-D`, so `set -euo pipefail` never fails on a clippy
  warning — the script "passes" dirty, contradicting "if housekeeping fails, fix it". Add
  `-D warnings` (or `-D` on the chosen lints) so a dirty tree exits non-zero.
- Add the missing trailing newline.

`CLAUDE.md`:
- The "Always Run Housekeeping" line points at
  `/home/isaak/RustroverProjects/pixelconnector` — **wrong project**. Fix to this repo's
  path (`/home/isaak/RustroverProjects/pathfinding`) now that `bin/housekeeping.sh` actually
  lives here, or it'll keep misdirecting.

## Out of scope (future, not "minor")
- Contour LOD / fewer levels when zoomed out (visual clutter + segment count at full
  zoom-out). Separate feature.
- Zoom-to-cursor instead of zoom-to-center.

## Verification
- After #1: zoom fully out — terrain fills in over a few frames with no single-frame stall;
  `LoadedChunks` count climbs then plateaus at the whole map; zoom in, it drops.
- After #2: (manual) despawning the player doesn't crash the HUD.
- After #3: launch and watch the first frames — no giant/!flicker scale bar.
- `cargo build` clean; `./bin/housekeeping.sh` runs under bash and **fails** if any clippy
  warning is present (then is clean).
