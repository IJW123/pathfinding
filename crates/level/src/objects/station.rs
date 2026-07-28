//! Station derivation: stations are not authored — a finished track earns one at each terminal
//! that has a loading/unloading place (a warehouse, for now) within reach. The pose comes from the
//! track itself, so the platform sits on the rail by construction. Lives in `level`, not `station`:
//! `station` has no `rail` dependency, while `level` already composes both — and runtime rail
//! building (manual or AI-routed) can call this same seam later.

use bevy::prelude::*;

use rail::track::RailTrack;
use station::bundle::station as station_bundle;
use station::constants::STATION_RADIUS;

/// The stations `track` earns: one at each terminal with a storage position within
/// [`STATION_RADIUS`]. A terminal near no loading/unloading place gets none. A degenerate
/// single-pose track (length 0) contributes one terminal, not two stacked stations.
#[must_use]
pub fn terminal_stations(track: &RailTrack, storage_positions: &[Vec2]) -> Vec<impl Bundle> {
    let mut terminals = vec![0.0];
    if track.length() > 0.0 {
        terminals.push(track.length());
    }
    terminals
        .into_iter()
        .map(|s| track.sample(s))
        .filter(|pose| {
            storage_positions
                .iter()
                .any(|storage| storage.distance(pose.position) <= STATION_RADIUS)
        })
        .map(|pose| station_bundle(pose.position, pose.angle, STATION_RADIUS))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use station::components::Station;

    /// A long straight track: its ends are far enough apart (1000 > 2 × `STATION_RADIUS`) that a
    /// storage can be in range of one terminal and not the other.
    fn long_track() -> RailTrack {
        RailTrack::new(vec![Vec2::ZERO, Vec2::new(1000.0, 0.0)])
    }

    /// Spawn the derived bundles into a bare world and collect each station's position.
    fn station_positions(track: &RailTrack, storages: &[Vec2]) -> Vec<Vec2> {
        let mut world = World::new();
        for bundle in terminal_stations(track, storages) {
            world.spawn(bundle);
        }
        world
            .query_filtered::<&Transform, With<Station>>()
            .iter(&world)
            .map(|tf| tf.translation.xy())
            .collect()
    }

    #[test]
    fn terminal_with_storage_in_range_earns_a_station_at_its_pose() {
        let positions = station_positions(&long_track(), &[Vec2::new(0.0, 50.0)]);
        let [pos] = positions[..] else {
            panic!("one station, got {positions:?}");
        };
        assert!(pos.distance(Vec2::ZERO) < 1e-3, "on the start terminal");
    }

    #[test]
    fn terminal_near_nothing_earns_none() {
        assert!(station_positions(&long_track(), &[Vec2::new(500.0, 500.0)]).is_empty());
        assert!(station_positions(&long_track(), &[]).is_empty());
    }

    #[test]
    fn both_terminals_covered_earn_two() {
        let storages = [Vec2::new(0.0, 50.0), Vec2::new(1000.0, 50.0)];
        let positions = station_positions(&long_track(), &storages);
        assert_eq!(positions.len(), 2);
        assert!(positions.iter().any(|p| p.distance(Vec2::ZERO) < 1e-3));
        assert!(
            positions
                .iter()
                .any(|p| p.distance(Vec2::new(1000.0, 0.0)) < 1e-3)
        );
    }
}
