# p25 — Sprite + auto-hitbox pipeline

## Goal
Spawn objects that carry **both** a PNG sprite and a collider, where the collider is a
**convex-hull approximation of the sprite's opaque pixels**, generated automatically (no
hand-authoring per PNG).

## Decisions (locked)
- **Bake location:** offline tool. PNG is never read by runtime world logic.
- **Hull shape:** convex hull (uses existing `Collider::convex`). Convex-only for now.
- **Scaling:** per-object world size. Bake emits a *normalized* hull (longest image side = 1.0,
  centered on image center, y-up). Spawn multiplies by a world-size knob — mirrors the existing
  `scaled_hull` / `TRIANGLE_SIZE` pattern. Sprite `custom_size` derives from the **same** factor
  so sprite and hull always align.
- Zoom scaling is free: sprites/colliders live in world space; camera zoom handles it. Not part
  of this work.

## Architecture rule being protected
Render depends on world logic, never the reverse. A PNG is a render asset; a hitbox is world
logic. So:
- **Offline bake tool** reads PNG → emits hull data (a `.ron` manifest). Build-time only, outside
  the runtime dep graph.
- **World logic** builds `Collider` from the manifest's hull points — reads a *data file*, not an
  image.
- **Render** loads the PNG via `AssetServer` and attaches a `Sprite`.
- The two sides are joined only by a shared `SpriteId` key (a newtype string), not a crate dep.

## Coordinate / normalization contract (the part that must not drift)
The `Sprite` draws the **full** texture rect, centered on the entity. So the hull must be
normalized in that exact frame:
- origin = image center
- y flipped to y-up
- divide by `longest_side = max(img_w, img_h)` in pixels
- `hull_pt = ((px - w/2) / longest, (h/2 - py) / longest)`

At spawn, given `world_size` (world units for the longest side):
- `collider = Collider::convex(hull_pts * world_size)?` — **fallible**: `Collider::convex` returns
  `Result<_, DegenerateHullError>` (see `hitboxes_rapier`). Unlike `obstacle::scaled_hull`, whose
  input is in-code unit constants safe to `.expect`, the hull here comes from an external data file
  that can drift or be hand-mangled. So `collider_for` either returns `Result` or `.expect`s with a
  message naming the offending `SpriteId` — caller-facing, not an authoring assert.
- The runtime only has `aspect` (= img_w / img_h) from the manifest, **not** the raw pixel dims, so
  re-express the sprite size in those terms (longest side = `world_size`, shorter side scaled by
  aspect):
  - landscape (`aspect >= 1`): `custom_size = (world_size, world_size / aspect)`
  - portrait  (`aspect < 1`):  `custom_size = (world_size * aspect, world_size)`
  This is algebraically identical to `(img_w, img_h) * (world_size / longest)` but implementable
  from manifest data alone.

The scaling math lives in **one** place (`sprites` crate) so hull and sprite cannot diverge.

## New crates
### `crates/spritebake` (offline bin)
- Deps: `image`, `serde`, `ron`. No bevy, no runtime crates.
- Scans `assets/sprites/*.png` (sprite id = file stem).
- Per image: alpha-threshold opaque pixels → convex hull (monotone chain, ~20 lines; no parry
  dep needed — `Collider::convex` re-hulls at runtime so this is just manifest compaction) →
  normalize per contract above.
- Alpha threshold is a named const in the bin (e.g. `ALPHA_OPAQUE = 128`), not a magic literal.
- **Degenerate guard:** if an image yields < 3 non-collinear opaque points (fully transparent, a
  single dot, a 1px line), the hull is degenerate and `Collider::convex` would later error. The bake
  must fail loudly *here* — print the offending file and exit non-zero — so a bad asset never reaches
  the runtime manifest. Don't silently skip.
- Emits `assets/sprite_manifest.ron`. On-disk hull points are **plain `(f32, f32)` tuples**, not
  `Vec2` — `spritebake` has no bevy/glam dep, and the `sprites` side deserializes into a tuple repr
  too (see below) to avoid needing the glam `serde` feature:
  ```
  ( sprites: {
      "warehouse": ( image_path: "sprites/warehouse.png", aspect: 1.33, hull: [(x, y), ...] ),
  })
  ```
- Run manually: `cargo run -p spritebake`. Note in plan: re-bake when art changes (manifest can drift).

### `crates/sprites` (world-logic side, shared catalog)
- Deps: `bevy`, `hitboxes_rapier`, `serde`, `ron`.
- Files (no logic in mod.rs):
  - `components.rs` — `SpriteId(String)` newtype (derive `Clone, Hash, Eq, PartialEq`),
    `SpriteRef { id: SpriteId, world_size: f32 }` component.
  - `manifest.rs` — **serde-facing** types matching the `.ron` exactly: `RawSpriteDef { image_path,
    aspect, hull: Vec<(f32, f32)> }` + the top-level `{ sprites }` wrapper. Kept separate from the
    in-memory catalog so the file format and the runtime type can evolve independently and we don't
    drag the glam `serde` feature in.
  - `catalog.rs` — `SpriteDef { image_path: String, aspect: f32, hull: Vec<Vec2> }` (tuples mapped to
    `Vec2` on load), `SpriteCatalog` resource (`HashMap<SpriteId, SpriteDef>`).
  - `scale.rs` — single source of scaling math. `collider_for(def, world_size) -> Result<Collider,
    DegenerateHullError>` (or an `.expect` naming the `SpriteId` — see contract); `sprite_size(def,
    world_size) -> Vec2`. Unit-test both, incl. portrait vs landscape aspect.
  - `loader.rs` — `load_catalog` system (PreStartup): `std::fs` read of `assets/sprite_manifest.ron`
    → parse `RawSpriteDef`s → build resource. **Missing/unparseable manifest:** `warn!` and leave the
    catalog empty rather than panic — first run before any bake, or a render-only build, shouldn't
    crash the app. A later `collider_for` on a missing id is the thing that errors, with a clear key.
  - `plugin.rs` — `SpritesPlugin`: inserts empty catalog, adds `load_catalog` to PreStartup.
  - `constants.rs` — manifest path.

## Modified crates
- **workspace `Cargo.toml`**: add `png` to bevy features (PNG decode for texture loading — verify the
  exact feature name against the pinned bevy 0.18 metadata before relying on it); add `image`,
  `serde`, `ron` to `[workspace.dependencies]`. Sprite *rendering* itself already works (the codebase
  uses `bevy::sprite_render` today) — only image decoding is missing.
- **`level`**: depend on `sprites`. `spawn_level` gains a `catalog: Res<SpriteCatalog>` param and
  spawns the first sprite object eagerly:
  `(transform, <markers>, catalog.collider_for(id, size)?, SpriteRef { id, world_size })`.
  Collider built eagerly (catalog loaded at PreStartup) → no 1-frame gap for collision.
  - **Pick the first target deliberately.** The storage building is the *worst* first target:
    `logistics::storage_building` hardcodes `Collider::obb(half_extents)`, so giving it a hull means
    either overwriting that component (a quick-fix smell, and `half_extents` becomes dead) or
    parameterizing the bundle's collider. Prefer wiring a **fresh, standalone static obstacle**
    (warehouse-as-scenery) first — no existing bundle owns its collider, so the end-to-end path is
    clean. Retrofitting `storage_building` to take a `Collider` arg is a separate, later change.
- **`render`**: depend on `sprites`. New `sprite/` module: `attach_sprite_texture` on
  `Added<SpriteRef>` (Update, mirroring the existing `Added<…>` attach systems) →
  `Sprite { image: asset_server.load(path), custom_size: sprite_size(...) }`.
  - Add `Without<SpriteRef>` only to the mesh-attach systems whose entities can actually carry a
    `SpriteRef`. For phase 4 (standalone obstacle target) that's `attach_obstacle_mesh`. The
    `Storage`/`DockZone` systems need it only once a *storage* building gets a sprite — note it,
    don't pre-emptively touch them.
- **`app`**: add `SpritesPlugin`. No explicit ordering needed — PreStartup always runs before
  Startup regardless of plugin add order, so the catalog is ready before `spawn_level`.

## Phasing
1. **Bake tool + manifest** — run on `warehouse.png`, eyeball the hull point count/shape.
2. **`sprites` crate** — catalog, `SpriteRef`, scaling math, loader. Unit test scaling math.
3. **render texture attach + `png` feature** — verify a textured quad renders.
4. **Wire one object end-to-end** — a fresh standalone static obstacle textured with `warehouse`
   (not the storage building; see Modified crates). Run app, confirm sprite and hull line up (toggle
   a debug draw of the collider if needed).
5. **(Later, separate)** migrate player + obstacles; decide placeholder strategy for objects with
   no art (they keep procedural meshes — `SpriteRef` is additive, non-breaking).

## Out of scope / noted limits
- **Concave sprites** over-approximate. If needed later: add a `Compound` variant to
  `ColliderShape` + convex decomposition in the bake tool. Not now.
- No hot-reload of the manifest; re-run `spritebake` after art changes.
- Objects without `SpriteRef` render exactly as today.
- **Manifest staleness is unguarded.** Editing a PNG without re-baking silently desyncs hull from
  art. Acceptable for now (single dev), but note it — a baked hash/mtime check is the eventual fix.

<!-- auto-reviewed -->
