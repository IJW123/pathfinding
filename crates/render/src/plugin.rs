use bevy::app::{App, Plugin};

use crate::elevation::plugin::ContourRenderPlugin;
use crate::logistics::plugin::StorageRenderPlugin;
use crate::obstacle::plugin::ObstacleRenderPlugin;
use crate::player::plugin::PlayerRenderPlugin;
use crate::selection::plugin::SelectionRenderPlugin;
use crate::sprite::plugin::SpriteTexturePlugin;

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ContourRenderPlugin,
            ObstacleRenderPlugin,
            StorageRenderPlugin,
            PlayerRenderPlugin,
            SelectionRenderPlugin,
            SpriteTexturePlugin,
        ));
    }
}
