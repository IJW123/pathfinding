use bevy::app::{App, Plugin};

use crate::elevation::plugin::ContourRenderPlugin;
use crate::obstacle::plugin::ObstacleRenderPlugin;
use crate::player::plugin::PlayerRenderPlugin;

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ContourRenderPlugin,
            ObstacleRenderPlugin,
            PlayerRenderPlugin,
        ));
    }
}
