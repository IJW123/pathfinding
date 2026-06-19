use bevy::prelude::*;

use camera_main::plugin::CameraMainPlugin;
use collision_rapier::plugin::{CollisionPlugin, CollisionSet};
use hud::plugin::HudPlugin;
use level::plugin::LevelPlugin;
use logistics::plugin::LogisticsPlugin;
use motion::plugin::{MotionPlugin, MotionSet};
use render::plugin::RenderPlugin;
use selection::plugin::SelectionPlugin;
use sprites::plugin::SpritesPlugin;
use world::elevation::plugin::ElevationPlugin;

fn main() {
    App::new()
        // Assets live at the workspace-root `assets/`, but bevy resolves its asset root against the
        // running package's dir (`crates/app`) under `cargo run`. Point it back two levels so the
        // `AssetServer` and the sprite manifest loader (cwd-relative) agree on one `assets/` dir.
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            file_path: "../../assets".to_string(),
            ..default()
        }))
        .add_plugins((
            CameraMainPlugin,
            CollisionPlugin,
            MotionPlugin,
            SelectionPlugin,
            ElevationPlugin,
            LogisticsPlugin,
            SpritesPlugin,
            LevelPlugin,
            HudPlugin,
            RenderPlugin,
        ))
        .configure_sets(FixedUpdate, MotionSet.after(CollisionSet))
        .run();
}
