use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::window::PrimaryWindow;

use crate::components::CoordReadout;
use player::components::Player;
use world::elevation::height_fn::HeightFn;

pub fn spawn_hud(mut commands: Commands) {
    commands.spawn((
        Text2d::new("x: 0.0  y: 0.0  z: 0.0"),
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(Color::srgb(0.95, 0.95, 0.95)),
        Anchor::TOP_RIGHT,
        Transform::from_xyz(0.0, 0.0, 10.0),
        CoordReadout,
    ));
}

#[expect(clippy::type_complexity, reason = "Bevy query filters")]
pub fn update_coord_text(
    height: Res<HeightFn>,
    player: Single<&Transform, (With<Player>, Without<CoordReadout>, Without<Camera2d>)>,
    camera: Single<&Transform, (With<Camera2d>, Without<CoordReadout>, Without<Player>)>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut readout: Query<(&mut Text2d, &mut Transform), With<CoordReadout>>,
) {
    let pos = player.translation.truncate();
    let z = height.sample(pos);
    let cam_pos = camera.translation.truncate();
    let half = Vec2::new(window.width(), window.height()) * 0.5;
    let corner = cam_pos + Vec2::new(half.x - 8.0, half.y - 8.0);
    for (mut text, mut transform) in &mut readout {
        text.0 = format!("x: {:.1}  y: {:.1}  z: {:.1}", pos.x, pos.y, z);
        transform.translation.x = corner.x;
        transform.translation.y = corner.y;
    }
}
