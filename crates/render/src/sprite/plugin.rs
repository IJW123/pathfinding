use bevy::app::{App, Plugin, Update};
use bevy::prelude::*;

use sprites::catalog::SpriteCatalog;
use sprites::components::SpriteRef;
use sprites::scale::sprite_size;

/// Render side of the sprite pipeline: gives each `SpriteRef` entity its PNG texture, sized to match
/// the collider the world side built from the same `SpriteRef`. Render-only — it reads the catalog
/// for the image path and loads the texture; the world side never touches the image.
pub struct SpriteTexturePlugin;

impl Plugin for SpriteTexturePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, attach_sprite_texture);
    }
}

/// One-shot at spawn: attach the texture for each newly-added `SpriteRef`. An id missing from the
/// catalog is a `warn!` (no texture) rather than a crash — the world side already reported it when
/// its collider lookup failed.
fn attach_sprite_texture(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    catalog: Res<SpriteCatalog>,
    query: Query<(Entity, &SpriteRef), Added<SpriteRef>>,
) {
    for (entity, sprite_ref) in &query {
        let Some(def) = catalog.get(&sprite_ref.id) else {
            warn!(
                "SpriteRef '{}' has no catalog entry; entity left untextured",
                sprite_ref.id.as_str()
            );
            continue;
        };
        commands.entity(entity).insert(Sprite {
            image: asset_server.load(&def.image_path),
            custom_size: Some(sprite_size(def, sprite_ref.world_size)),
            ..default()
        });
    }
}
