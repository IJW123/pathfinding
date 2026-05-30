# P4 Motion Refactor

## Goals

- Make `motion` safer to use for entities beyond the player.
- Document exactly what `MeasuredVelocity` represents.
- Preserve the leaf-crate shape of `motion` unless a stronger integration need appears.
- Add focused tests for velocity measurement, initialization, scheduling, and HUD freshness.

## Current State

`motion` measures velocity from `Transform` delta over the previous frame:

```text
velocity = (current_xy - previous_xy) / delta_seconds
```

The player currently opts in by spawning with:

- `MeasuredVelocity::default()`
- `PrevPosition(spawn)`

The app schedules `MotionSet` after `CollisionSet`, and HUD updates after `MotionSet`, so the displayed speed reflects resolved world displacement after collision response.

## Improvement Plan

### 1. Lock Down Semantics

Define `MeasuredVelocity` as resolved world displacement per second.

It includes:

- player movement
- terrain-modified movement
- collision push-out
- scripted `Transform` changes
- any other transform correction that runs before `MotionSet`

It does not specifically mean input speed or requested movement speed.

Document this on:

- `MeasuredVelocity`
- `MotionPlugin`
- `MotionSet`

### 2. Make Initialization Safer

The current manual pattern is easy to misuse. `PrevPosition::default()` is `Vec2::ZERO`, so tracked entities spawned away from origin can report a first-frame speed spike.

Preferred fix:

- initialize `PrevPosition` from the current `Transform` automatically when velocity tracking is added

Possible approaches:

- an `OnAdd<MeasuredVelocity>` observer
- a startup/update initialization system for entities missing `PrevPosition`
- a small public bundle/helper that requires an initial position

Preferred direction is automatic initialization because it keeps spawners simple and makes future entities harder to wire incorrectly.

### 3. Handle Zero-Delta Frames

Current behavior returns early when `dt <= 0.0`, leaving `PrevPosition` stale.

Desired behavior:

- set measured velocity to zero
- sync `PrevPosition` to the current transform

This prevents movement during a zero-delta frame from being accumulated into a later nonzero frame as fake velocity.

### 4. Clarify Scheduling Contract

Keep `motion` as a leaf crate for now. Do not make it depend on `collision` just to enforce ordering internally.

Document the scheduling contract:

```text
all movement/resolution systems whose displacement should count
-> MotionSet
-> consumers such as HUD
```

The current app ordering is correct:

```text
move_player -> CollisionSet -> MotionSet -> update_hud_text
```

Risk: another app can add `MotionPlugin` without configuring `MotionSet` after its movement and resolution systems. Documentation should make this explicit.

### 5. Decide Teleport And Reset Behavior

By default, transform teleports will appear as large measured velocities. That is consistent with literal transform delta, but may not be wanted for respawns or scripted warps.

Do not solve this immediately unless teleporting becomes part of gameplay.

Possible later addition:

- a reset component/event/system that syncs `PrevPosition` to `Transform` and zeros `MeasuredVelocity`

## Test Plan

### Unit Tests For `motion`

#### Measures Basic Velocity

Entity starts at `(10, 20)`, previous position is `(10, 20)`, then transform moves to `(13, 24)` with `dt = 1.0`.

Expected:

```text
velocity = (3, 4)
speed = 5
```

#### Uses Delta Time Correctly

Same displacement over `dt = 0.5`.

Expected:

```text
velocity = (6, 8)
speed = 10
```

#### No First-Frame Spike

Spawn an entity away from origin with tracking initialized from its current transform. Run motion once without moving it.

Expected:

```text
velocity = (0, 0)
```

#### Updates Previous Position After Measuring

Move once, measure velocity, move again, measure again.

Expected:

```text
second velocity is based only on the second movement delta
```

It must not measure from the original spawn point.

#### Zero Delta Time Does Not Accumulate Stale Motion

Move entity during a zero-delta frame, run motion, then run a normal frame without additional movement.

Expected:

```text
velocity = (0, 0)
```

The zero-delta movement should not be reported later as stale velocity.

### Integration Tests

#### Collision-Corrected Velocity Is Reflected

Run an app schedule where movement pushes the player into a wall, collision resolution adjusts the final position, then `MotionSet` runs.

Expected:

```text
MeasuredVelocity reflects final resolved displacement, not raw input displacement.
```

#### HUD Reads Fresh Velocity

Run a schedule with:

```text
movement -> MotionSet -> update_hud_text
```

Expected:

```text
HUD speed uses the current frame's measured velocity.
```

It should not display the previous frame's value.

## Suggested Implementation Order

1. Update docs for `MeasuredVelocity`, `MotionPlugin`, and `MotionSet`.
2. Add safe initialization for tracked entities.
3. Adjust zero-delta behavior.
4. Add motion unit tests.
5. Add one focused collision-order integration test.
6. Add one HUD freshness integration or schedule test.
