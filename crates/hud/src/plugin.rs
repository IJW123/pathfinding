use bevy::app::{App, Plugin, Startup, Update};

use crate::systems::{spawn_hud, update_hud_text, update_scale_bar};

/// HUD reads `MeasuredVelocity` across schedules (motion runs in `FixedUpdate`), so it sees
/// the last completed tick's value — no intra-schedule ordering needed.
pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_hud)
            .add_systems(Update, (update_hud_text, update_scale_bar));
    }
}
