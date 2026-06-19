use std::cmp::Ordering;

use bevy::prelude::*;

use hitboxes_rapier::components::{Collider, Static};
use hitboxes_rapier::convert::{transform_to_pose, vec2_to_parry};
use world::elevation::height_field::HeightField;
use world::terrain_effects::slope_speed::slope_speed_multiplier;

use crate::components::{Selectable, Selected};
use crate::constants::CONTROL_SPEED;

/// Left-click picks the topmost [`Selectable`] under the cursor and makes it the sole [`Selected`]
/// entity; a click on empty ground deselects. A click while the cursor sits outside the window
/// (no cursor position) is ignored — it must not clear the current selection.
pub fn select_on_click(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    camera: Single<(&Camera, &GlobalTransform), With<Camera2d>>,
    window: Single<&Window>,
    selectables: Query<(Entity, &Transform, &Collider), With<Selectable>>,
    selected: Query<Entity, With<Selected>>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let (cam, cam_transform) = *camera;
    let Ok(world_point) = cam.viewport_to_world_2d(cam_transform, cursor) else {
        return;
    };

    // Selectables are top-level entities, so `Transform` is their world pose — reuse the
    // existing transform→isometry seam rather than hand-rolling one.
    let point = vec2_to_parry(world_point);
    let hit = selectables
        .iter()
        .filter(|(_, transform, collider)| {
            collider
                .shape
                .to_shared_shape()
                .contains_point(&transform_to_pose(transform), point)
        })
        .max_by(|(_, a, _), (_, b, _)| {
            a.translation
                .z
                .partial_cmp(&b.translation.z)
                .unwrap_or(Ordering::Equal)
        })
        .map(|(entity, _, _)| entity);

    for entity in &selected {
        commands.entity(entity).remove::<Selected>();
    }
    if let Some(entity) = hit {
        commands.entity(entity).insert(Selected);
    }
}

/// Arrow keys move the [`Selected`] entity, slowed/sped by terrain slope. Generalized from the old
/// `move_player`: control is now whatever is selected. Runs before collision so the per-tick step
/// stays bounded (no tunneling). Writes `Transform` only.
///
/// `Static` entities are excluded: an immovable building can be selected (for readouts) but can't be
/// driven — the collision solver never corrects a static body, so driving one would phase it through
/// walls and other statics.
pub fn move_selected(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    height: Res<HeightField>,
    mut query: Query<&mut Transform, (With<Selected>, Without<Static>)>,
) {
    let mut direction = Vec2::ZERO;
    if keyboard.pressed(KeyCode::ArrowUp) {
        direction.y += 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowDown) {
        direction.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowLeft) {
        direction.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowRight) {
        direction.x += 1.0;
    }

    if direction != Vec2::ZERO {
        let dir = direction.normalize();
        for mut transform in &mut query {
            let grad = height.gradient(transform.translation.xy());
            let slope_mul = slope_speed_multiplier(dir, grad);
            let delta = dir * CONTROL_SPEED * slope_mul * time.delta_secs();
            transform.translation.x += delta.x;
            transform.translation.y += delta.y;
        }
    }
}
