use bevy::prelude::*;

use crate::components::{Solid, Static};
use crate::events::CollisionEvent;

pub fn resolve_solid_collisions(
    mut reader: MessageReader<CollisionEvent>,
    solids: Query<(), With<Solid>>,
    statics: Query<(), With<Static>>,
    mut transforms: Query<&mut Transform>,
) {
    for ev in reader.read() {
        let CollisionEvent {
            a,
            b,
            normal,
            depth,
        } = *ev;

        if !(solids.contains(a) && solids.contains(b)) {
            continue;
        }

        // `push = normal * depth` points a→b. Re-signed factors for that convention: the dynamic
        // body(s) move so the pair separates (a opposite the normal, b along it).
        let (a_factor, b_factor) = match (statics.contains(a), statics.contains(b)) {
            (true, true) => continue,
            (true, false) => (0.0, 1.0),
            (false, true) => (-1.0, 0.0),
            (false, false) => (-0.5, 0.5),
        };

        let push = normal * depth;
        let mut apply = |entity: Entity, factor: f32| {
            if factor == 0.0 {
                return;
            }
            if let Ok(mut t) = transforms.get_mut(entity) {
                t.translation.x += push.x * factor;
                t.translation.y += push.y * factor;
            }
        };
        apply(a, a_factor);
        apply(b, b_factor);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dynamic_a_pushed_opposite_normal() {
        let mut app = App::new();
        app.add_message::<CollisionEvent>()
            .add_systems(Update, resolve_solid_collisions);

        let a = app.world_mut().spawn((Transform::IDENTITY, Solid)).id();
        let b = app
            .world_mut()
            .spawn((Transform::from_xyz(1.0, 0.0, 0.0), Solid, Static))
            .id();

        app.world_mut()
            .resource_mut::<Messages<CollisionEvent>>()
            .write(CollisionEvent {
                a,
                b,
                normal: Vec2::X,
                depth: 0.5,
            });
        app.update();

        // a dynamic, b static, normal +X depth 0.5 ⇒ a moves −0.5 on X, b unmoved.
        let a_x = app
            .world()
            .entity(a)
            .get::<Transform>()
            .unwrap()
            .translation
            .x;
        let b_x = app
            .world()
            .entity(b)
            .get::<Transform>()
            .unwrap()
            .translation
            .x;
        assert!((a_x - -0.5).abs() < 1e-4, "a_x {a_x}");
        assert!((b_x - 1.0).abs() < 1e-4, "b_x {b_x}");
    }
}
