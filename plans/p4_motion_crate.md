# P4 — `motion/` crate + HUD speed indicator

Goal: a general "measured kinematic state" crate that observes
`Transform` deltas for any tagged entity, plus a HUD line showing
the player's measured speed.

---

## Why a new crate, not an addition to `collision/` or `player/`

- `collision/` is overlap detection + response. Kinematic measurement
  is a different concern; bundling them blurs the name.
- `player/` would scope the abstraction to one entity, defeating the
  point of asking ("we'll have other moving things").
- `motion/` is small now (~40 LOC) but is the natural home for future
  kinematic state: acceleration, max-speed caps, friction, smoothed
  velocity, last-direction-faced, etc. Same pattern as `hud/` —
  small focused crate beats stretching an existing one.

## Target layout

```
crates/motion/
  Cargo.toml
  src/
    lib.rs         # pub mod components; pub mod plugin; pub mod systems;
    components.rs  # MeasuredVelocity, PrevPosition
    plugin.rs      # MotionPlugin + MotionSet
    systems.rs     # measure_velocity
```

## Dep DAG update

```
app
 ├─ render  → world, player
 ├─ hud     → world, player, motion          (NEW edge)
 ├─ player  → world, collision, motion       (NEW edge — to insert components at spawn)
 ├─ motion                                   (NEW leaf, bevy only)
 ├─ world   → collision
 └─ collision
```

`motion/` is a leaf — bevy dep only, nothing else. Keeps it reusable
for any future moving entity (NPCs, projectiles, particles).

---

## Components

```rust
// crates/motion/src/components.rs
use bevy::prelude::*;

/// Velocity observed from Transform delta in the previous frame, in
/// world units per second. Read-only output of MotionPlugin — writing
/// to it has no effect (gets overwritten every frame).
#[derive(Component, Default, Debug, Clone, Copy)]
pub struct MeasuredVelocity(pub Vec2);

/// Internal bookkeeping for MeasuredVelocity. Stores last frame's
/// xy translation so the system can compute Δpos/Δt.
#[derive(Component, Default, Debug, Clone, Copy)]
pub struct PrevPosition(pub Vec2);
```

Both are zero-cost defaults — entities opt in by inserting both at
spawn. First-frame velocity will be `(pos - Vec2::ZERO) / dt` which is
wrong; cheapest fix is initializing `PrevPosition` to the spawn position
at the call site (one line per spawn).

Open question to confirm: do we want `MeasuredVelocity` to be a
`pub Vec2` field or a getter-only struct? Field is simpler; nothing
stops a system from writing to it, but writes are pointless (system
overwrites). Going with the field unless you disagree.

## System

```rust
// crates/motion/src/systems.rs
use bevy::prelude::*;
use crate::components::{MeasuredVelocity, PrevPosition};

pub fn measure_velocity(
    time: Res<Time>,
    mut q: Query<(&Transform, &mut PrevPosition, &mut MeasuredVelocity)>,
) {
    let dt = time.delta_secs();
    if dt <= 0.0 { return; }
    for (transform, mut prev, mut vel) in &mut q {
        let now = transform.translation.xy();
        vel.0 = (now - prev.0) / dt;
        prev.0 = now;
    }
}
```

## Plugin

```rust
// crates/motion/src/plugin.rs
use bevy::prelude::*;
use crate::systems::measure_velocity;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MotionSet;

pub struct MotionPlugin;

impl Plugin for MotionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, measure_velocity.in_set(MotionSet));
    }
}
```

## Ordering — the whole point of "measured"

### The constraint and why it exists

The reading should reflect *everything* that moved the entity this
frame — input, then collision pushback. So the chain must be:

```
move_player  →  CollisionSet (rebuild → detect → resolve)  →  measure_velocity
```

If `measure_velocity` ran *before* collision response, the frame where
the player walks into a wall would register their input velocity
(pushback hasn't happened yet), then *next* frame the pushback shows up
as velocity-away-from-the-wall. Off-by-one and wrong-signed at the
moment of impact. Running it after collision response gives the honest
"net displacement this frame / dt" reading.

### What "before/after" means here mechanically

`measure_velocity` reads `Transform.translation.xy()` at the moment it
runs and subtracts the value it stored last time it ran. `prev` always
holds end-of-frame position from the previous measurement; `vel =
(now - prev) / dt` is displacement between consecutive measurements.
Anything that mutates `Transform` *between* collision and measurement
gets folded into the reading. Anything that mutates it *after*
measurement doesn't. So measurement should run as late as possible in
game-state updates but before display reads (the HUD).

### Where to put the constraint

Two ways to express `MotionSet.after(CollisionSet)`:

- **Inside `MotionPlugin::build`** — forces a `motion → collision` dep
  edge just to import `CollisionSet`. Breaks motion's leaf-crate
  property and tangles two unrelated concerns.
- **In `app/src/main.rs`** — both sets already visible there. Motion
  stays a leaf. Same logic as why `move_player.before(CollisionSet)`
  lives in `PlayerPlugin` (player already depends on collision),
  inverted: when there's no dep edge to lean on, the constraint goes
  in the crate that imports both, i.e. `app/`.

Going with the second:
```rust
// app/src/main.rs
.configure_sets(Update, MotionSet.after(CollisionSet))
```

`move_player` is already `.before(CollisionSet)`, so transitively
`MotionSet` runs after both.

### HUD ordering

Without an explicit constraint, Bevy can interleave `update_hud_text`
and `measure_velocity` arbitrarily — meaning the speed readout might
be one frame stale on some frames and current on others depending on
scheduler choices. Imperceptible visually but non-deterministic. Cheap
to nail down:

```rust
// crates/hud/src/plugin.rs
.add_systems(Update, update_hud_text.after(MotionSet))
```

Hud already depends on `motion` for the component, so the import is
free. This lives in `HudPlugin` (not `app/`) — hud has the dep edge.

### Schedule choice — Update vs PostUpdate

Considered putting `measure_velocity` in `PostUpdate` to implicitly
guarantee it runs after all `Update` Transform mutations. Rejecting:

- The "runs after movement" guarantee becomes invisible at the call
  site (it's just "it's in a later schedule").
- Mixed scheduling makes future ordering questions harder to reason
  about.
- `PostUpdate` is conventionally engine plumbing
  (`TransformSystems::Propagate`, visibility, etc.); inserting
  gameplay measurement there muddies the line.

Stay in `Update` with explicit `.after(CollisionSet)`. If we later
need `GlobalTransform` (e.g. parented entities), the upgrade is to
switch from `Transform` to `GlobalTransform` *and* move to `PostUpdate`
so propagation has run — discrete future change, not something to
pre-pay for.

### Final ordering graph

```
Update:
  move_player.before(CollisionSet)
  CollisionSet { rebuild_spatial_hash → detect_collisions → resolve_solid_collisions }
  MotionSet.after(CollisionSet)   { measure_velocity }
  update_hud_text.after(MotionSet)
```

## Player integration

`crates/player/Cargo.toml`: add `motion = { workspace = true }`.

`crates/player/src/systems.rs::setup_player`: insert
`MeasuredVelocity::default()` + `PrevPosition(spawn_xy)` on the player
entity. One-line change.

No change to `move_player`.

---

## HUD speed indicator

`crates/hud/Cargo.toml`: add `motion = { workspace = true }`.

Layout decision: **append a second line to the existing `CoordReadout`
text** (vs. spawning a separate `Text2d` entity). Reasons:
- Single anchor point — no separate transform to keep aligned.
- One system update instead of two.
- If we want to split later, trivial.

Modified `update_coord_text` signature gains
`Single<&MeasuredVelocity, With<Player>>` and the format becomes:

```rust
text.0 = format!(
    "x: {:.1}  y: {:.1}  z: {:.1}\nspeed: {:.1}",
    pos.x, pos.y, z, vel.0.length(),
);
```

Displaying magnitude only — matches what "speed" means literally.
If we want the vector too later, easy add.

System rename: `update_coord_text` → `update_hud_text` (it's no longer
just coords). Component `CoordReadout` → `HudReadout`. Small cleanup;
say no if you'd rather leave the names.

Ordering: HUD update should run after `MotionSet` so it reads
fresh-this-frame velocity. Cheap to enforce:
```rust
.add_systems(Update, update_hud_text.after(MotionSet))
```

---

## Workspace wiring

`Cargo.toml` root:
- Add `"crates/motion"` to `members`.
- Add `motion = { path = "crates/motion" }` to `workspace.dependencies`.

`crates/app/Cargo.toml`: add `motion = { workspace = true }`.
`crates/app/src/main.rs`: add `MotionPlugin` to the plugin tuple.

---

## Step order (small, each compiles)

1. Create `crates/motion/` skeleton (Cargo.toml, lib.rs, empty modules).
   Wire into workspace + add `MotionPlugin` to `app/`. Verify build.
2. Implement `MeasuredVelocity` / `PrevPosition` / `measure_velocity` +
   `MotionSet` ordering. Build still green, system runs over empty query.
3. Insert components on player at spawn. Build + run — velocity is
   updated but unread.
4. HUD: add `motion` dep, query velocity, format second line, order
   after `MotionSet`. Build + run — speed shows up.
5. Housekeeping pass.

## Things deliberately **not** in scope

- Smoothing / low-pass filter on the displayed speed.
- Acceleration component.
- Max-speed caps, friction.
- Velocity-aware collision (CCD, etc).
- Generalizing the HUD to display arbitrary moving entities.

All easy to add later on top of `MeasuredVelocity`; none needed now.

## Open follow-up

First-frame velocity for entities that don't initialize `PrevPosition`
to their spawn position will spike to `spawn_pos / dt`. Acceptable for
now (player explicitly initializes; nothing else uses it yet). If we
later have many spawners, consider an `OnAdd<MeasuredVelocity>` observer
that auto-seeds `PrevPosition` from the current `Transform`. Noting,
not doing.
