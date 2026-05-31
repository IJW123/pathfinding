use bevy::prelude::*;

use crate::systems::{pan_camera, spawn_camera};

pub struct CameraMainPlugin;

impl Plugin for CameraMainPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera)
            .add_systems(Update, pan_camera);
    }
}
