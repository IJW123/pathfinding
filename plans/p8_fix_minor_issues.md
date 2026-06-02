# Plan: Minor fixes from the terrain-scale review

## Context

Cleanup pass on smaller issues found reviewing the zoom + scale-bar work (`p6`). None are
architectural (that's `p7`); these are latent panics, an Inf path, and tooling/doc drift.
Independent of each other and of `p7` (no shared files) — land in any order.

> The original "per-frame chunk-load budget" item was dropped: `p7` removes chunk streaming
> entirely (static tiles built at launch), so there's no per-frame load to budget.

## 1. `Single` → `Option<Single>` where the entity can vanish

**Problem:** `update_hud_text` uses `Single<… With<Player>>`; `Single` panics on zero-or-many
matches. The moment the player despawns (death/respawn) the HUD system crashes.

**Fix:** `crates/hud/src/systems.rs` → `update_hud_text`: take
`player: Option<Single<(&Transform, &MeasuredVelocity), With<Player>>>` and early-return
(implicit) when `None`. Camera `Single`s (`zoom_camera`, `update_scale_bar`) are lower risk —
the camera is spawned once at startup and never despawned — so leave them unless/until a
second camera or teardown appears. Flagging, not changing.

## 2. Scale-bar divide-by-zero / first-frame Inf

**Problem:** `update_scale_bar` computes `mpp = ortho.area.width() / window.width()` then
fill width `= nice / mpp`. Bevy computes `ortho.area` in `PostUpdate`, so on the first
`Update` `area` can be zero → `mpp = 0` → width `= Inf` → one frame of a garbage-width node.

**Fix:** `crates/hud/src/systems.rs` → `update_scale_bar`: guard before writing —
`if window.width() > 0.0 && mpp.is_finite() && mpp > 0.0` (else leave the bar as-is this
frame). Cosmetic, but removes a real NaN/Inf path.

## 3. Order projection readers after the zoom writer (low priority)

**Problem:** `zoom_camera` writes `Projection`; `update_scale_bar` reads it with no ordering →
the bar may use last-frame scale (≤1-frame lag after a zoom). Imperceptible, nondeterministic.

**Fix (optional):** order `update_scale_bar.after(zoom_camera)` (cross-plugin, so in `app`'s
`configure_sets` or via a shared set). Skip if not worth the wiring; documented so it's a
known choice, not an oversight.

## 4. Housekeeping script + CLAUDE.md drift

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
- After #1: (manual) despawning the player doesn't crash the HUD.
- After #2: launch and watch the first frames — no giant/flickering scale bar.
- `cargo build` clean; `./bin/housekeeping.sh` runs under bash and **fails** if any clippy
  warning is present (then is clean).
