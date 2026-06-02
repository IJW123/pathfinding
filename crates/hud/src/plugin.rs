use bevy::app::{App, Plugin, Startup, Update};
use bevy::prelude::IntoScheduleConfigs;

use motion::plugin::MotionSet;

use crate::systems::{spawn_hud, update_hud_text, update_scale_bar};

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_hud)
            .add_systems(Update, (update_hud_text.after(MotionSet), update_scale_bar));
    }
}
