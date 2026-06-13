Add: 
- Different terrain features
~~- SAT Collision detection~~
- Structures such as buildings, roads, rails
- Moving objects such as ~~simple polygons~~ NPCs cars, trains
- Sprite system that renders different sprites based on elevation (angle), terrain type
- Economic structures
- Currency system
- Market system
- Macro for inserting new objects

## Design note: collision filtering (pass-through / flight)

Future need: entities that overlap in x,y but don't collide (e.g. a plane flying
over a blocking box). Decision deferred; recorded here so it doesn't evaporate.

**Dependency rule (non-negotiable):** `world` depends on `collision` (walls/obstacles
attach `Collider`/`Solid`), so collision can never read a type defined in `world` —
that's a cycle. The filtering *mechanism* (component type + pair test) lives in
`collision` as semantics-free data; `world`/gameplay own the *semantics* and write
the values, ideally via one sync system deriving them from real game state
(same pattern as planned `WallState -> Solid`).

**Option A — collision layers/masks.** `CollisionLayers { memberships, filters }`
bitmask component; pair collides iff masks intersect. World defines what the bits
mean (GROUND/AIR/...). Standard (Rapier/Avian/Box2D), one u32 test, also covers
categorical cases (ghosts, ally-skipping projectiles, player-only triggers).
Cost: modal, not spatial — takeoff/landing become layer-swap state sync; wrong
layer at the wrong moment = clipping bugs at transitions.

**Option B — height bands.** `ZBand { min, max }` per collider; pair collides iff
bands overlap (one interval check next to the AABB test). Plane at altitude 60 vs
0–20 box: disjoint, no pair. Transitions free — descending re-enables collision at
exactly the right altitude, no mode flips. Fits the existing elevation system.
Cost: every collider carries a band (all z=0 today); can't express categorical
exceptions.

**Lean:** B for flight (height is real in this world; layers would fake it).
A and B are complementary, not rivals — add A only when categorical filtering
shows up. A `pass_through` marker defined in `world` and interpreted by collision
is vetoed (dependency cycle).