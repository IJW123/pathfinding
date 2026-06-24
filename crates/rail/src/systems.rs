use bevy::prelude::*;

use selection::components::Selected;

use crate::components::{Locomotive, RailHeading, TrackPosition};
use crate::constants::{DRIVE_KEY, LOCO_SPEED, LOCO_Z, TURN_KEY};
use crate::track::RailTrack;

/// Filter for the controllable locomotive: the one that is both a `Locomotive` and currently
/// `Selected`. Aliased to keep the driver queries readable and past clippy's `type_complexity` gate.
type SelectedLoco = (With<Locomotive>, With<Selected>);

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
    track: Single<&RailTrack>,
    mut loco: Query<(&mut TrackPosition, &RailHeading), SelectedLoco>,
) {
    if keyboard.pressed(DRIVE_KEY) {
        let length = track.length();
        for (mut position, heading) in &mut loco {
            let delta = LOCO_SPEED * time.delta_secs() * heading.sign();
            position.0 = (position.0 + delta).clamp(0.0, length);
        }
    }
}

/// Project every locomotive's [`TrackPosition`] onto the rail, writing its world `Transform`. Runs
/// unconditionally (not gated on selection) so the loco always sits on the track. Facing follows the
/// travel direction: the tangent for `Forward`, the tangent + π for `Backward`.
pub fn project_locomotive(
    track: Single<&RailTrack>,
    mut loco: Query<(&TrackPosition, &RailHeading, &mut Transform), With<Locomotive>>,
) {
    for (position, heading, mut transform) in &mut loco {
        let pose = track.sample(position.0);
        transform.translation = pose.position.extend(LOCO_Z);
        let facing = pose.angle + heading.facing_offset();
        transform.rotation = Quat::from_rotation_z(facing);
    }
}
