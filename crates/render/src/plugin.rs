use bevy::app::{App, Plugin};

use crate::elevation::plugin::ContourRenderPlugin;
use crate::obstacle::plugin::ObstacleRenderPlugin;
use crate::player::plugin::PlayerRenderPlugin;
use crate::wall::plugin::WallRenderPlugin;

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ContourRenderPlugin,
            WallRenderPlugin,
            ObstacleRenderPlugin,
            PlayerRenderPlugin,
        ));
    }
}
