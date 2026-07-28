use bevy::prelude::*;

use crate::bundle::{locomotive, rail_track};
use crate::components::RailHeading;
use crate::track::RailTrack;

/// The entities a [`spawn_rail_line`] call created: the track and the locomotive riding it. Returned
/// so callers can wire more onto either — e.g. give the loco an `Inventory` to make it a cargo train.
#[derive(Debug, Clone, Copy)]
pub struct RailLine {
    pub track: Entity,
    pub loco: Entity,
}

/// Spawn a rail line: the track entity plus a locomotive riding it at `start_s`/`heading`. Returns
/// both ([`RailLine`]).
///
/// The ordering is deliberate and the reason this lives in one place. The track id is reserved up
/// front (`spawn_empty`) so the loco can carry `OnTrack(id)` while `track` is still borrowable:
/// `RailTrack` is not `Clone`, [`locomotive`] borrows `&track` to seed its start pose, and
/// [`rail_track`] *consumes* the track — so the consuming insert must come last. Open-coding this at
/// each call site is exactly the kind of subtle borrow dance that drifts into a bug.
pub fn spawn_rail_line(
    commands: &mut Commands,
    track: RailTrack,
    start_s: f32,
    heading: RailHeading,
) -> RailLine {
    let track_entity = commands.spawn_empty().id();
    let loco = commands
        .spawn(locomotive(&track, track_entity, start_s, heading))
        .id();
    commands.entity(track_entity).insert(rail_track(track));
    RailLine {
        track: track_entity,
        loco,
    }
}
