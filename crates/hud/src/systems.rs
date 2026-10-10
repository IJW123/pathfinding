use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::components::{HudReadout, ScaleBarFill, ScaleBarLabel};
use crate::constants::TARGET_BAR_PX;
use crate::format::{format_distance, nice_distance};
use measurement::motion::components::MeasuredVelocity;
use selection::components::Selected;
use world::elevation::height_field::HeightField;

pub fn update_hud_text(
    height: Res<HeightField>,
    selected: Option<Single<(&Transform, &MeasuredVelocity), With<Selected>>>,
    mut readout: Single<&mut Text, With<HudReadout>>,
) {
    if let Some(selected) = selected {
        let (transform, vel) = *selected;
        let pos = transform.translation.truncate();
        let z = height.sample(pos);
        let speed = vel.0.length();
        readout.0 = format!(
            "x: {}  y: {}\nz: {:.1} m\nspeed: {:.1} m/s",
            format_distance(pos.x),
            format_distance(pos.y),
            z,
            speed,
        );
    }
}

/// Size the scale bar to the largest "nice" distance fitting within
/// [`TARGET_BAR_PX`](crate::constants::TARGET_BAR_PX) at the current zoom, and
/// label it. Reads the camera's computed visible area, so it tracks any zoom.
pub fn update_scale_bar(
    projection: Single<&Projection, With<Camera2d>>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut fill: Single<&mut Node, With<ScaleBarFill>>,
    mut label: Single<&mut Text, With<ScaleBarLabel>>,
) {
    if let Projection::Orthographic(ortho) = projection.into_inner() {
        let metres_per_px = ortho.area.width() / window.width();
        if metres_per_px.is_finite() && metres_per_px > 0.0 {
            let nice = nice_distance(TARGET_BAR_PX * metres_per_px);
            fill.width = Val::Px(nice / metres_per_px);
            label.0 = format_distance(nice);
        }
    }
}
