use bevy::app::{App, Plugin, Startup, Update};
use bevy::scene::SpawnListSystem;

use crate::scene::hud;
use crate::systems::{update_hud_text, update_scale_bar};

/// HUD reads `MeasuredVelocity` across schedules (motion runs in `FixedUpdate`), so it sees
/// the last completed tick's value — no intra-schedule ordering needed.
pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, hud.spawn())
            .add_systems(Update, (update_hud_text, update_scale_bar));
    }
}
