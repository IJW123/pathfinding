# P4 — Motion crate hardening

Trimmed follow-up on the shipped `motion/` crate (built per
`p4_motion_crate.md`). Supersedes `p4_motion_refactor.md`, dropping the
speculative items it promoted out of that plan's "not in scope" list.

## Scope

**In:**
1. Document semantics on `MeasuredVelocity` / `MotionPlugin` / `MotionSet`.
2. Document the scheduling contract (the real leaf-crate footgun).
3. Zero-delta handling — one-liner correctness fix.
4. Unit tests for `measure_velocity` (crate shipped with none).

**Deferred (not now):**
- §2 auto-init of `PrevPosition`. Moot while the only tracked entity
  (player) spawns at `Vec2::ZERO`. Revisit when a second, off-origin
  moving entity lands — and prefer a public spawn **bundle/helper** over
  a hidden `OnAdd` observer at that time.
- §5 teleport/reset behavior. No teleport gameplay exists yet.
- Integration tests (collision-corrected, HUD freshness). They mostly
  assert Bevy honors `.after()`; cost (standing up world+collision+player
  harness) outweighs the value of testing a documented ordering contract.

---

## 1. Semantics docs

Rewrite the doc comment on `MeasuredVelocity` (components.rs) to state it
is **resolved world displacement / second**, folding in everything that
mutates `Transform` before `MotionSet`: player input, terrain-modified
movement, collision push-out, scripted transform writes. Explicitly *not*
requested/input speed.

Add a crate-level summary to `MotionPlugin` (plugin.rs) doc comment.

## 2. Scheduling contract docs

On `MotionSet` (plugin.rs), document the ordering requirement — this is
the actual hazard, since nothing enforces it inside a leaf crate:

```text
<all movement + resolution systems whose displacement should count>
  -> MotionSet
  -> consumers (e.g. HUD)
```

State plainly: an app that adds `MotionPlugin` without ordering
`MotionSet` after its movement/resolution systems gets stale/wrong
readings. Note the current app wiring is correct
(`move_player -> CollisionSet -> MotionSet -> update_hud_text`) and that
the constraint lives in `app/main.rs` by design (leaf crate can't import
`CollisionSet` without breaking its leaf property — see
`p4_motion_crate.md` §Ordering).

## 3. Zero-delta fix

`crates/motion/src/systems.rs::measure_velocity`. Replace the early
`return` on `dt <= 0.0` with per-entity handling that zeros velocity and
still syncs `PrevPosition`, so a zero-dt frame can't strand a stale
`prev` that later reports as fake velocity:

```rust
pub fn measure_velocity(
    time: Res<Time>,
    mut query: Query<(&Transform, &mut PrevPosition, &mut MeasuredVelocity)>,
) {
    let dt = time.delta_secs();
    for (transform, mut prev, mut vel) in &mut query {
        let now = transform.translation.xy();
        vel.0 = if dt > 0.0 { (now - prev.0) / dt } else { Vec2::ZERO };
        prev.0 = now;
    }
}
```

(Also drops the early return — aligns with CLAUDE.md's single-exit pref.)

## 4. Unit tests

New `#[cfg(test)]` module in `systems.rs`. Build a minimal `App` (or set
`Time` manually) per case, spawn entity with `Transform` + `PrevPosition`
+ `MeasuredVelocity`, advance `Time`, run `measure_velocity`, assert.

Cases:
- **basic velocity** — prev `(10,20)`, move to `(13,24)`, `dt=1.0`
  → `vel == (3,4)`, `len == 5`.
- **dt scaling** — same delta, `dt=0.5` → `vel == (6,8)`, `len == 10`.
- **no first-frame spike** — entity at `(100,100)`, `PrevPosition(100,100)`,
  no move, run once → `vel == (0,0)`.
- **prev updates** — move, measure, move again, measure → second reading
  reflects only the second delta, not displacement from spawn.
- **zero-delta no stale accumulation** — move during `dt==0` frame, run;
  then normal frame with no further move → `vel == (0,0)`.

---

## Open question (raise, don't block)

`p4_motion_crate.md` left `MeasuredVelocity` as a writable `pub Vec2`.
Docs now call it read-only. If you want that enforced rather than
documented, that's a getter-only refactor — out of scope here unless you
say otherwise.

## Implementation order (each compiles)

1. §3 zero-delta fix + §4 unit tests (the only behavior change; tests
   guard it).
2. §1 + §2 docs.
3. Housekeeping pass (`cargo build`/`clippy`, fix warnings).

Touched files: `crates/motion/src/{systems.rs, components.rs, plugin.rs}`.
No changes to `app/`, `player/`, or `hud/`.
