use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::window::PrimaryWindow;

use crate::components::HudReadout;
use motion::components::MeasuredVelocity;
use player::components::Player;
use world::elevation::height_fn::HeightFn;

pub fn spawn_hud(mut commands: Commands) {
    commands.spawn((
        Text2d::new("x: 0.0  y: 0.0  z: 0.0\nspeed: 0.0"),
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(Color::srgb(0.95, 0.95, 0.95)),
        Anchor::TOP_RIGHT,
        Transform::from_xyz(0.0, 0.0, 10.0),
        HudReadout,
    ));
}

#[expect(clippy::type_complexity, reason = "Bevy query filters")]
pub fn update_hud_text(
    height: Res<HeightFn>,
    player: Single<
        (&Transform, &MeasuredVelocity),
        (With<Player>, Without<HudReadout>, Without<Camera2d>),
    >,
    camera: Single<&Transform, (With<Camera2d>, Without<HudReadout>, Without<Player>)>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut readout: Query<(&mut Text2d, &mut Transform), With<HudReadout>>,
) {
    let (player_tx, vel) = *player;
    let pos = player_tx.translation.truncate();
    let z = height.sample(pos);
    let speed = vel.0.length();
    let cam_pos = camera.translation.truncate();
    let half = Vec2::new(window.width(), window.height()) * 0.5;
    let corner = cam_pos + Vec2::new(half.x - 8.0, half.y - 8.0);
    for (mut text, mut transform) in &mut readout {
        text.0 = format!(
            "x: {:.1}  y: {:.1}  z: {:.1}\nspeed: {:.1}",
            pos.x, pos.y, z, speed,
        );
        transform.translation.x = corner.x;
        transform.translation.y = corner.y;
    }
}
