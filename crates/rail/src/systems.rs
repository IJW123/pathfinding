use bevy::prelude::*;

use selection::components::Selected;

use crate::components::{Locomotive, OnTrack, RailHeading, TrackPosition};
use crate::constants::{DRIVE_KEY, LOCO_SPEED, LOCO_Z, TURN_KEY};
use crate::track::RailTrack;

/// Filter for the controllable locomotive: the one that is both a `Locomotive` and currently
/// `Selected`. Aliased to keep the driver queries readable and past clippy's `type_complexity` gate.
type SelectedLoco = (With<Locomotive>, With<Selected>);

/// Read-side of a locomotive for projection: its arc-length, heading, the track it rides, and the
/// transform to write. Aliased to keep `project_locomotive`'s query under clippy's `type_complexity`.
type LocoProjection = (
    &'static TrackPosition,
    &'static RailHeading,
    &'static OnTrack,
    &'static mut Transform,
);

/// Flip the selected locomotive's heading on the turn key. Reads edge-detected input, so it lives in
/// `Update`: under `FixedUpdate`'s variable run-count a `just_pressed` could fire zero or two times
/// per frame, dropping or self-cancelling the flip.
pub fn turn_locomotive(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut loco: Query<&mut RailHeading, SelectedLoco>,
) {
    if keyboard.just_pressed(TURN_KEY) {
        for mut heading in &mut loco {
            *heading = heading.flipped();
        }
    }
}

/// Advance the selected locomotive's arc-length while the drive key is *held*. A held key is a level,
/// not an edge, so it's safe under `FixedUpdate`'s variable run-count. Clamped to the track ends —
/// dead stop, no wrap. Writes [`TrackPosition`] only; [`project_locomotive`] turns it into a pose.
pub fn drive_locomotive(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    tracks: Query<&RailTrack>,
    mut loco: Query<(&mut TrackPosition, &RailHeading, &OnTrack), SelectedLoco>,
) {
    if keyboard.pressed(DRIVE_KEY) {
        for (mut position, heading, on_track) in &mut loco {
            // Resolve this loco's own track; an orphaned ref (track despawned) idles it.
            let Ok(track) = tracks.get(on_track.0) else {
                continue;
            };
            let delta = LOCO_SPEED * time.delta_secs() * heading.sign();
            position.0 = (position.0 + delta).clamp(0.0, track.length());
        }
    }
}

/// Project every locomotive's [`TrackPosition`] onto the rail, writing its world `Transform`. Runs
/// unconditionally (not gated on selection) so the loco always sits on the track. Facing follows the
/// travel direction: the tangent for `Forward`, the tangent + π for `Backward`.
pub fn project_locomotive(
    tracks: Query<&RailTrack>,
    mut loco: Query<LocoProjection, With<Locomotive>>,
) {
    for (position, heading, on_track, mut transform) in &mut loco {
        // Each loco samples its own track; an orphaned ref leaves it at its last pose.
        let Ok(track) = tracks.get(on_track.0) else {
            continue;
        };
        let pose = track.sample(position.0);
        transform.translation = pose.position.extend(LOCO_Z);
        let facing = pose.angle + heading.facing_offset();
        transform.rotation = Quat::from_rotation_z(facing);
    }
}

#[cfg(test)]
mod tests {
    use bevy::ecs::system::RunSystemOnce;
    use bevy::prelude::*;

    use crate::bundle::{locomotive, rail_track};
    use crate::components::{Locomotive, OnTrack, RailHeading};
    use crate::constants::LOCO_Z;
    use crate::track::RailTrack;

    use super::project_locomotive;

    /// Spawn a track entity plus a loco riding it, mirroring `spawn_rail_line`'s reserve-then-insert
    /// ordering with direct `World` calls (so the test owns the entity ids). Returns the track entity.
    fn spawn_line(world: &mut World, points: Vec<Vec2>, start_s: f32) -> Entity {
        let track = RailTrack::new(points);
        let track_entity = world.spawn_empty().id();
        world.spawn(locomotive(
            &track,
            track_entity,
            start_s,
            RailHeading::Forward,
        ));
        world.entity_mut(track_entity).insert(rail_track(track));
        track_entity
    }

    /// Regression guard for the `Single<&RailTrack>` bug: two tracks must coexist, each locomotive
    /// projected onto *its own* track. Before per-loco `OnTrack` association, a second track silently
    /// disabled the projector entirely.
    #[test]
    fn each_loco_projects_onto_its_own_track() {
        let mut world = World::new();

        // Two tracks with deliberately unalike geometry: a mixed-up lookup lands on the wrong axis
        // and fails loudly rather than coincidentally passing.
        let track_h = spawn_line(
            &mut world,
            vec![Vec2::new(0.0, 0.0), Vec2::new(200.0, 0.0)],
            50.0,
        );
        let track_v = spawn_line(
            &mut world,
            vec![Vec2::new(0.0, 500.0), Vec2::new(0.0, 700.0)],
            50.0,
        );

        world
            .run_system_once(project_locomotive)
            .expect("project_locomotive runs with only query params");

        // Each loco's projected world position must sit on the track its `OnTrack` names.
        let mut q = world.query_filtered::<(&OnTrack, &Transform), With<Locomotive>>();
        let mut seen = 0;
        for (on_track, transform) in q.iter(&world) {
            let pos = transform.translation;
            assert!((pos.z - LOCO_Z).abs() < 1e-4, "loco rides the loco Z layer");
            if on_track.0 == track_h {
                assert!(
                    pos.truncate().distance(Vec2::new(50.0, 0.0)) < 1e-3,
                    "horizontal-track loco at arc-length 50 sits at (50, 0), got {pos:?}"
                );
            } else if on_track.0 == track_v {
                assert!(
                    pos.truncate().distance(Vec2::new(0.0, 550.0)) < 1e-3,
                    "vertical-track loco at arc-length 50 sits at (0, 550), got {pos:?}"
                );
            } else {
                panic!("loco references an unknown track entity");
            }
            seen += 1;
        }
        assert_eq!(seen, 2, "both locomotives projected");
    }
}
