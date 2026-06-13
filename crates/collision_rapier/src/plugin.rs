use bevy::prelude::*;

use crate::events::CollisionEvent;
use crate::mapping::ColliderMap;
use crate::physics::PhysicsWorld;
use crate::solver::resolve_collisions;
use crate::step::step_collision_pipeline;
use crate::sync::sync_physics_world;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CollisionSet;

/// Collision runs in `FixedUpdate`: a fixed tick bounds the per-step displacement
/// (max speed ÷ tick rate), which is what makes thin walls tunnel-proof.
pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PhysicsWorld>()
            .init_resource::<ColliderMap>()
            .add_message::<CollisionEvent>()
            .add_systems(
                FixedUpdate,
                (
                    sync_physics_world,
                    step_collision_pipeline,
                    resolve_collisions,
                )
                    .chain()
                    .in_set(CollisionSet),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mapping::BodyBinding;
    use bevy::ecs::message::Messages;
    use hitboxes_rapier::components::{Collider, Solid, Static};

    fn app() -> App {
        let mut app = App::new();
        app.init_resource::<PhysicsWorld>()
            .init_resource::<ColliderMap>()
            .add_message::<CollisionEvent>()
            .add_systems(
                Update,
                (
                    sync_physics_world,
                    step_collision_pipeline,
                    resolve_collisions,
                )
                    .chain(),
            );
        app
    }

    fn spawn_box(app: &mut App, pos: Vec2, half: f32, is_static: bool) -> Entity {
        let mut e = app.world_mut().spawn((
            Transform::from_xyz(pos.x, pos.y, 0.0),
            Collider::obb(Vec2::splat(half)),
            Solid,
        ));
        if is_static {
            e.insert(Static);
        }
        e.id()
    }

    fn drain_events(app: &mut App) -> Vec<(Entity, Entity, Vec2, f32)> {
        app.world_mut()
            .resource_mut::<Messages<CollisionEvent>>()
            .drain()
            .map(|e| (e.a, e.b, e.normal, e.depth))
            .collect()
    }

    fn translation(app: &App, entity: Entity) -> Vec2 {
        app.world()
            .get::<Transform>(entity)
            .expect("transform")
            .translation
            .truncate()
    }

    #[test]
    fn sync_lifecycle_spawn_and_despawn() {
        let mut app = app();
        let dynamic = spawn_box(&mut app, Vec2::ZERO, 10.0, false);
        let fixed = spawn_box(&mut app, Vec2::new(100.0, 0.0), 10.0, true);
        app.update();
        {
            let map = app.world().resource::<ColliderMap>();
            let physics = app.world().resource::<PhysicsWorld>();
            assert_eq!(map.bindings.len(), 2);
            assert_eq!(physics.colliders.len(), 2);
            assert_eq!(physics.bodies.len(), 0, "all colliders are parentless");
            assert!(matches!(
                map.bindings.get(&fixed),
                Some(BodyBinding::Static { .. })
            ));
            assert!(matches!(
                map.bindings.get(&dynamic),
                Some(BodyBinding::Dynamic { .. })
            ));
        }

        app.world_mut().entity_mut(dynamic).despawn();
        app.update();
        let map = app.world().resource::<ColliderMap>();
        let physics = app.world().resource::<PhysicsWorld>();
        assert_eq!(map.bindings.len(), 1);
        assert_eq!(physics.colliders.len(), 1);
    }

    #[test]
    fn changed_collider_and_static_transform_propagate() {
        let mut app = app();
        let fixed = spawn_box(&mut app, Vec2::ZERO, 10.0, true);
        app.update();

        app.world_mut()
            .entity_mut(fixed)
            .insert(Collider::circle(7.0));
        app.world_mut()
            .entity_mut(fixed)
            .insert(Transform::from_xyz(50.0, -20.0, 0.0));
        app.update();

        let map = app.world().resource::<ColliderMap>();
        let physics = app.world().resource::<PhysicsWorld>();
        let Some(BodyBinding::Static { collider }) = map.bindings.get(&fixed) else {
            panic!("static binding expected");
        };
        let collider = physics.colliders.get(*collider).expect("live handle");
        assert!(
            (collider.shared_shape().as_ball().expect("ball").radius - 7.0).abs() < 1e-6,
            "shape change propagated"
        );
        assert!(
            (collider.position().translation.x - 50.0).abs() < 1e-4
                && (collider.position().translation.y + 20.0).abs() < 1e-4,
            "transform change propagated"
        );
    }

    #[test]
    fn static_static_overlap_ignored() {
        let mut app = app();
        let a = spawn_box(&mut app, Vec2::ZERO, 10.0, true);
        let b = spawn_box(&mut app, Vec2::new(5.0, 0.0), 10.0, true); // deeply overlapping statics
        app.update();
        assert!(drain_events(&mut app).is_empty());
        assert_eq!(translation(&app, a), Vec2::ZERO);
        assert_eq!(translation(&app, b), Vec2::new(5.0, 0.0));
    }

    #[test]
    fn dynamic_pushed_out_of_static_with_event() {
        let mut app = app();
        let dynamic = spawn_box(&mut app, Vec2::ZERO, 10.0, false);
        let fixed = spawn_box(&mut app, Vec2::new(15.0, 0.0), 10.0, true);
        app.update();

        let events = drain_events(&mut app);
        assert_eq!(events.len(), 1);
        let (a, b, normal, depth) = events[0];
        assert_eq!((a, b), (dynamic, fixed), "dynamic side is a");
        assert!(normal.x > 0.9, "normal a→b (dyn→static)");
        assert!((depth - 5.0).abs() < 1e-3, "pre-resolution depth");
        let pos = translation(&app, dynamic);
        assert!(pos.x < -4.0, "pushed left out of overlap, got {pos:?}");
        assert_eq!(translation(&app, fixed), Vec2::new(15.0, 0.0));
    }

    #[test]
    fn dynamic_pair_pushes_apart_evenly() {
        let mut app = app();
        let a = spawn_box(&mut app, Vec2::ZERO, 10.0, false);
        let b = spawn_box(&mut app, Vec2::new(15.0, 0.0), 10.0, false);
        app.update();

        assert_eq!(drain_events(&mut app).len(), 1, "dynamic-dynamic fires");
        let (pa, pb) = (translation(&app, a), translation(&app, b));
        assert!(pa.x < 0.0 && pb.x > 15.0, "both moved apart: {pa:?} {pb:?}");
        assert!(
            (pa.x + (pb.x - 15.0)).abs() < 1e-3,
            "split evenly: {pa:?} {pb:?}"
        );
    }

    #[test]
    fn rotated_static_obb_pushes_out() {
        let mut app = app();
        // 45°-rotated static square; dynamic overlaps its upper-left edge region.
        let fixed = app
            .world_mut()
            .spawn((
                Transform::from_xyz(20.0, 0.0, 0.0)
                    .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_4)),
                Collider::obb(Vec2::splat(10.0)),
                Solid,
                Static,
            ))
            .id();
        let dynamic = spawn_box(&mut app, Vec2::ZERO, 8.0, false);
        app.update();

        assert_eq!(drain_events(&mut app).len(), 1);
        let pos = translation(&app, dynamic);
        assert!(pos.x < 0.0, "pushed away from rotated box, got {pos:?}");
        assert_eq!(translation(&app, fixed), Vec2::new(20.0, 0.0));
    }

    #[test]
    fn off_center_convex_pushes_circle() {
        let mut app = app();
        // Hull occupying local x∈[0,20], y∈[-10,10] — centroid right of the transform.
        app.world_mut().spawn((
            Transform::IDENTITY,
            Collider::convex(vec![
                Vec2::new(0.0, -10.0),
                Vec2::new(20.0, -10.0),
                Vec2::new(20.0, 10.0),
                Vec2::new(0.0, 10.0),
            ])
            .expect("valid hull"),
            Solid,
            Static,
        ));
        let dynamic = app
            .world_mut()
            .spawn((
                Transform::from_xyz(25.0, 0.0, 0.0),
                Collider::circle(8.0),
                Solid,
            ))
            .id();
        app.update();

        assert_eq!(drain_events(&mut app).len(), 1);
        let pos = translation(&app, dynamic);
        assert!(pos.x > 27.0, "circle pushed right of the hull, got {pos:?}");
    }

    #[test]
    fn near_pair_within_prediction_no_event() {
        let mut app = app();
        let dynamic = spawn_box(&mut app, Vec2::ZERO, 10.0, false);
        // Gap of 4u < prediction distance (8): in the pair graph, but not touching.
        spawn_box(&mut app, Vec2::new(24.0, 0.0), 10.0, true);
        app.update();

        let physics = app.world().resource::<PhysicsWorld>();
        assert_eq!(
            physics.narrow_phase.contact_graph().interactions().count(),
            1,
            "near-contact enters the pair graph"
        );
        assert!(drain_events(&mut app).is_empty(), "but no touching event");
        assert_eq!(translation(&app, dynamic), Vec2::ZERO, "and no correction");
    }

    #[test]
    fn non_solid_dynamic_reported_not_resolved() {
        let mut app = app();
        // Sensor-style dynamic: no Solid marker.
        let sensor = app
            .world_mut()
            .spawn((Transform::IDENTITY, Collider::obb(Vec2::splat(10.0))))
            .id();
        let fixed = spawn_box(&mut app, Vec2::new(15.0, 0.0), 10.0, true);
        app.update();

        let events = drain_events(&mut app);
        assert_eq!(events.len(), 1, "overlap still reported");
        assert_eq!((events[0].0, events[0].1), (sensor, fixed));
        assert_eq!(translation(&app, sensor), Vec2::ZERO, "but not corrected");
    }

    #[test]
    fn static_marker_flip_rebinds() {
        let mut app = app();
        let entity = spawn_box(&mut app, Vec2::ZERO, 10.0, false);
        app.update();
        assert!(matches!(
            app.world().resource::<ColliderMap>().bindings.get(&entity),
            Some(BodyBinding::Dynamic { .. })
        ));

        app.world_mut().entity_mut(entity).insert(Static);
        app.update();
        let map = app.world().resource::<ColliderMap>();
        let physics = app.world().resource::<PhysicsWorld>();
        assert!(matches!(
            map.bindings.get(&entity),
            Some(BodyBinding::Static { .. })
        ));
        assert_eq!(physics.colliders.len(), 1, "rebound, not duplicated");
    }
}
