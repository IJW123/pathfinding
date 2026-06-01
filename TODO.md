# TODO

## Elevation: gate chunk lifecycle on camera movement

`update_loaded_chunks` (`crates/world/src/elevation/chunk_lifecycle.rs`) runs every
`Update` frame: it rebuilds the desired-chunk set and scans every loaded key even when
the camera hasn't moved. Cheap at the current ~156-chunk scale, but it's needless churn.

**Fix idea:** skip the diff when the camera transform is unchanged since last run (e.g.
track last camera chunk-coord or position, early-return if equal). Keep it simple — no
change-detection plumbing beyond what's needed.

**Priority:** low. Do after the contour cache work, and re-check against
the chunk lifecycle weaknesses note before touching the load/unload path.

## Future: terrain type as an analytic field (outside render)

Terrain semantics (air / water / dirt / rock / road) should live in the `world` crate as
an analytic field beside `HeightFn` (e.g. `terrain_kind(pos) -> TerrainKind`), NOT in
render. Render stays a consumer that receives terrain info and decides how to draw it.
Downstream gameplay (movement gating: boats in water, flying entities in air, etc.) reads
the same field. Not scoped yet — captured so the contour cache work doesn't paint us into
a render-owns-terrain corner.
