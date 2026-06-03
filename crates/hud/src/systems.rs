use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::components::{HudReadout, ScaleBarFill, ScaleBarLabel};
use crate::constants::{
    HUD_FONT_SIZE, HUD_MARGIN_PX, HUD_TEXT_COLOR, SCALE_BAR_COLOR, SCALE_BAR_HEIGHT_PX,
    TARGET_BAR_PX,
};
use crate::format::{format_distance, nice_distance};
use motion::components::MeasuredVelocity;
use player::components::Player;
use world::elevation::height_field::HeightField;

pub fn spawn_hud(mut commands: Commands) {
    commands.spawn((
        Text::new("x: 0 m  y: 0 m\nz: 0.0 m\nspeed: 0.0 m/s"),
        TextFont {
            font_size: HUD_FONT_SIZE,
            ..default()
        },
        TextColor(HUD_TEXT_COLOR),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(HUD_MARGIN_PX),
            right: Val::Px(HUD_MARGIN_PX),
            ..default()
        },
        HudReadout,
    ));

    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(HUD_MARGIN_PX),
            left: Val::Px(HUD_MARGIN_PX),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexStart,
            row_gap: Val::Px(2.0),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                Text::new("0 m"),
                TextFont {
                    font_size: HUD_FONT_SIZE,
                    ..default()
                },
                TextColor(HUD_TEXT_COLOR),
                ScaleBarLabel,
            ));
            parent.spawn((
                Node {
                    width: Val::Px(TARGET_BAR_PX),
                    height: Val::Px(SCALE_BAR_HEIGHT_PX),
                    ..default()
                },
                BackgroundColor(SCALE_BAR_COLOR),
                ScaleBarFill,
            ));
        });
}

pub fn update_hud_text(
    height: Res<HeightField>,
    player: Option<Single<(&Transform, &MeasuredVelocity), With<Player>>>,
    mut readout: Single<&mut Text, With<HudReadout>>,
) {
    if let Some(player) = player {
        let (player_tx, vel) = *player;
        let pos = player_tx.translation.truncate();
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
