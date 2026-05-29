What LoadedChunks does

It's the registry mapping chunk coord → entity ID. Single source of truth for "which chunks are currently live."

Used by chunk_lifecycle::update_loaded_chunks for two checks each frame:

1. Skip already-loaded chunks. When iterating desired_chunks(...), loaded.0.contains_key(&coord) answers "do I need to spawn this?" — without this, we'd re-spawn every chunk every frame.
2. Find stale chunks. loaded.0.keys().filter(|k| !desired.contains(k)) — every loaded coord that is no longer in the desired set gets despawned and removed from the map.

The Entity value is needed because despawning is by entity ID, not by coord. Bevy doesn't index entities by component value, so we maintain this side index ourselves.

Nothing else reads it today. Future systems that want "is there a chunk at coord X?" or "give me the entity for coord X" will go through this resource.

  ---
Downsides of the three side effects

1. LoadedChunks is just coord → Entity (no mesh state)

Downside. It's lying about its name. "Loaded" implies the chunk is ready to use — but a system reacting at frame N+1 to a ChunkLoaded fired at frame N might query LoadedChunks, get an Entity, and find that the contour mesh isn't attached yet
(or worse, has been attached but the consumer's own components haven't). There's no per-chunk "readiness" tracking.

Concretely: pathfinding subscribes to ChunkLoaded, starts a cost-bake task that finishes 3 frames later. During those 3 frames, anything querying LoadedChunks thinks the chunk is "loaded" but pathfinding data isn't there. The map gives no way
to ask "is the bake done?"

Mitigation when it becomes a problem. Either marker components on the chunk entity (PathfindingBaked, ContourRendered) that consumers query, or a richer LoadedChunks value (enum { Spawning, Ready }). Don't pre-build either until the first
system actually needs it.

2. (update_loaded_chunks, render_contours_on_chunk_loaded).chain()

Downside. .chain() forces serial execution. Right now both systems are cheap, but as the consumer list grows (pathfinding_bake_on_chunk_loaded, physics_collider_on_chunk_loaded, audio_zone_on_chunk_loaded...), chaining all of them defeats
Bevy's parallel scheduler. They could all run in parallel after lifecycle since they touch disjoint components — but .chain() says "no, one at a time."

There's also a hidden cost: chain() implies command flushing between systems. The chunk entity is commands.spawn(...)'d in lifecycle; render reads ev.entity and commands.entity(ev.entity).insert(...). That works because chain flushes
commands, but it's a frame-latency band-aid for what should be event-driven.

The right fix later. Drop .chain() and accept one-frame latency on first appearance — the alternative is ApplyDeferred/SystemSet ordering where lifecycle's set runs first and all consumers run after in parallel. For a 2D contour overlay, one
frame of invisibility on a new chunk is invisible to the player; not worth serializing every consumer.

3. ChunkUnloaded emitted but unread (#[expect(dead_code)])

Downside. Dead API surface = lying API surface. Two specific risks:

- Wrong contract. Right now chunk_lifecycle despawns the entity and writes ChunkUnloaded in the same system. By the time a future reader reads the event next frame, commands.entity(ev.entity) is operating on a dead entity. In Bevy 0.18 that
  warns or panics depending on operation. So the event as currently emitted is unusable for cleanup — the consumer can't do anything that requires the entity. We won't know that until the first consumer hits it.
- Field drift. coord and entity are speculative — the first real consumer might want bounds: Aabb2d, or reason: enum { OutOfView, Manual }. Adding to a #[derive(Message)] struct is fine, but right now we have zero feedback on whether the
  shape is right.

The honest move. Either delete the event entirely until something needs it (YAGNI), or restructure lifecycle so the despawn happens in a separate system after consumers have processed the unload event — which means another .chain(), which
compounds problem #2.

  ---
The common thread: events + a coord→entity map are the right shape, but the lifecycle ordering is fragile. It works now because there's exactly one consumer (contour_render). The cracks will show with the second.