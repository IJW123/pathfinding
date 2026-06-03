use bevy::prelude::*;

use crate::components::{MeasuredVelocity, PrevPosition};

pub fn measure_velocity(
    time: Res<Time>,
    mut query: Query<(&Transform, &mut PrevPosition, &mut MeasuredVelocity)>,
) {
    let dt = time.delta_secs();
    for (transform, mut prev, mut vel) in &mut query {
        let now = transform.translation.xy();
        vel.0 = if dt > 0.0 {
            (now - prev.0) / dt
        } else {
            Vec2::ZERO
        };
        prev.0 = now;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    use std::time::Duration;

    fn world_with_dt(dt: f32) -> World {
        let mut world = World::new();
        let mut time = Time::<()>::default();
        time.advance_by(Duration::from_secs_f32(dt));
        world.insert_resource(time);
        world
    }

    fn advance(world: &mut World, dt: f32) {
        world
            .resource_mut::<Time<()>>()
            .advance_by(Duration::from_secs_f32(dt));
    }

    fn spawn(world: &mut World, prev: Vec2, pos: Vec2) -> Entity {
        world
            .spawn((
                Transform::from_xyz(pos.x, pos.y, 0.0),
                PrevPosition(prev),
                MeasuredVelocity::default(),
            ))
            .id()
    }

    fn run(world: &mut World) {
        world
            .run_system_once(measure_velocity)
            .expect("measure_velocity runs");
    }

    fn velocity(world: &World, entity: Entity) -> Vec2 {
        world
            .get::<MeasuredVelocity>(entity)
            .expect("entity has MeasuredVelocity")
            .0
    }

    fn move_to(world: &mut World, entity: Entity, pos: Vec2) {
        world
            .get_mut::<Transform>(entity)
            .expect("has Transform")
            .translation = pos.extend(0.0);
    }

    #[test]
    fn measures_basic_velocity() {
        let mut world = world_with_dt(1.0);
        let e = spawn(&mut world, Vec2::new(10.0, 20.0), Vec2::new(13.0, 24.0));
        run(&mut world);
        assert_eq!(velocity(&world, e), Vec2::new(3.0, 4.0));
        assert_eq!(velocity(&world, e).length(), 5.0);
    }

    #[test]
    fn uses_delta_time_correctly() {
        let mut world = world_with_dt(0.5);
        let e = spawn(&mut world, Vec2::new(10.0, 20.0), Vec2::new(13.0, 24.0));
        run(&mut world);
        assert_eq!(velocity(&world, e), Vec2::new(6.0, 8.0));
        assert_eq!(velocity(&world, e).length(), 10.0);
    }

    #[test]
    fn no_first_frame_spike() {
        let mut world = world_with_dt(1.0);
        let e = spawn(&mut world, Vec2::new(100.0, 100.0), Vec2::new(100.0, 100.0));
        run(&mut world);
        assert_eq!(velocity(&world, e), Vec2::ZERO);
    }

    #[test]
    fn previous_position_updates_after_measuring() {
        let mut world = world_with_dt(1.0);
        let e = spawn(&mut world, Vec2::ZERO, Vec2::new(1.0, 1.0));
        run(&mut world);
        assert_eq!(velocity(&world, e), Vec2::new(1.0, 1.0));

        move_to(&mut world, e, Vec2::new(3.0, 5.0));
        run(&mut world);
        // second reading is delta from (1,1), not from spawn (0,0)
        assert_eq!(velocity(&world, e), Vec2::new(2.0, 4.0));
    }

    #[test]
    fn zero_delta_does_not_accumulate_stale_motion() {
        let mut world = world_with_dt(0.0);
        // moved during a zero-delta frame
        let e = spawn(&mut world, Vec2::ZERO, Vec2::new(5.0, 5.0));
        run(&mut world);
        assert_eq!(velocity(&world, e), Vec2::ZERO);

        // normal frame, no further movement: must not report the prior delta
        advance(&mut world, 1.0);
        run(&mut world);
        assert_eq!(velocity(&world, e), Vec2::ZERO);
    }
}
