use bevy::prelude::*;

use hitboxes_rapier::components::Collider;

use crate::components::{Locomotive, RailHeading, TrackPosition};
use crate::constants::{LOCO_HALF_EXTENTS, LOCO_Z, TRACK_Z};
use crate::track::RailTrack;

/// The locomotive starting at arc-length `start_s` on `track`, facing `heading`. Its `Collider` is
/// for click-picking only (the loco is never `Solid`, so the solver detects but never pushes it).
/// The initial `Transform` is seeded from the track so it renders on-rail before the first
/// `project_locomotive` tick.
#[must_use]
pub fn locomotive(track: &RailTrack, start_s: f32, heading: RailHeading) -> impl Bundle {
    let pose = track.sample(start_s);
    let facing = pose.angle + heading.facing_offset();
    (
        Locomotive,
        Collider::obb(LOCO_HALF_EXTENTS),
        TrackPosition(start_s),
        heading,
        Transform::from_translation(pose.position.extend(LOCO_Z))
            .with_rotation(Quat::from_rotation_z(facing)),
    )
}

/// The track entity carrying the `RailTrack`. Its `Transform` sits at `TRACK_Z` so the rail mesh the
/// renderer attaches layers below obstacles and above the ground; the mesh vertices themselves are
/// world-space, so this transform only sets the Z layer.
#[must_use]
pub fn rail_track(track: RailTrack) -> impl Bundle {
    (track, Transform::from_xyz(0.0, 0.0, TRACK_Z))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_track() -> RailTrack {
        RailTrack::new(vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(100.0, 0.0),
            Vec2::new(100.0, 100.0),
        ])
    }

    #[test]
    fn locomotive_has_full_composition_and_sits_on_rail() {
        let track = sample_track();
        let mut world = World::new();
        let e = world
            .spawn(locomotive(&track, 0.0, RailHeading::Forward))
            .id();

        assert!(world.get::<Locomotive>(e).is_some());
        assert!(world.get::<Collider>(e).is_some(), "collider for picking");
        assert_eq!(world.get::<TrackPosition>(e).expect("seeded").0, 0.0);
        assert_eq!(
            *world.get::<RailHeading>(e).expect("seeded"),
            RailHeading::Forward
        );

        // Spawned on the rail at start: at s=0 that's the first authored point.
        let transform = world.get::<Transform>(e).expect("has transform");
        assert!(transform.translation.truncate().distance(Vec2::ZERO) < 1e-3);
        assert!((transform.translation.z - LOCO_Z).abs() < 1e-6);
    }

    #[test]
    fn rail_track_entity_layered_at_track_z() {
        let track = sample_track();
        let mut world = World::new();
        let e = world.spawn(rail_track(track)).id();
        assert!(world.get::<RailTrack>(e).is_some());
        assert!(
            (world.get::<Transform>(e).expect("transform").translation.z - TRACK_Z).abs() < 1e-6
        );
    }
}
