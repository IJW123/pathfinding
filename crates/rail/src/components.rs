use std::f32::consts::PI;

use bevy::prelude::*;

use selection::components::{FreeMoveExempt, Selectable};

/// A rail-bound locomotive. Selectable like any unit, but `FreeMoveExempt` so the generic arrow-key
/// mover skips it — its only motion comes from [`drive_locomotive`](crate::systems) along the track
/// named by its [`OnTrack`]. Several can coexist, one per track. Both required components are
/// parameter-free, so the constructor only inserts the parameterized pieces (track ref, collider,
/// track position, heading, transform).
#[derive(Component)]
#[require(Selectable, FreeMoveExempt)]
pub struct Locomotive;

/// The track entity this locomotive rides. The drive/project systems resolve the
/// [`RailTrack`](crate::track::RailTrack) by this id, so tracks and locomotives are many independent
/// pairs rather than one global track — what makes multiple rail lines possible.
#[derive(Component, Debug, Clone, Copy)]
pub struct OnTrack(pub Entity);

/// Arc-length position of the locomotive along its [`RailTrack`](crate::track::RailTrack), in world
/// units from the track's start. The loco's *only* degree of freedom — world `Transform` is derived
/// from this each tick, so the rail constraint is structural.
#[derive(Component, Debug, Clone, Copy)]
pub struct TrackPosition(pub f32);

/// Which way along the track increasing/decreasing arc-length the locomotive currently travels.
/// `Forward` advances arc-length; `Backward` retreats it. The turn button flips this.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum RailHeading {
    Forward,
    Backward,
}

impl RailHeading {
    /// Sign applied to the along-track step: `+1` forward, `-1` backward.
    #[must_use]
    pub fn sign(self) -> f32 {
        match self {
            Self::Forward => 1.0,
            Self::Backward => -1.0,
        }
    }

    /// The opposite heading.
    #[must_use]
    pub fn flipped(self) -> Self {
        match self {
            Self::Forward => Self::Backward,
            Self::Backward => Self::Forward,
        }
    }

    /// Radians added to the track tangent so the sprite faces its travel direction: `0` forward,
    /// `π` backward. The single source of this rule — the spawn bundle and the per-tick projector
    /// both read it, so the facing can't drift between them.
    #[must_use]
    pub fn facing_offset(self) -> f32 {
        match self {
            Self::Forward => 0.0,
            Self::Backward => PI,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flipped_is_involutive_and_negates_sign() {
        assert_eq!(RailHeading::Forward.flipped(), RailHeading::Backward);
        assert_eq!(RailHeading::Backward.flipped(), RailHeading::Forward);
        // Two flips return to the start, and a flip negates the along-track step direction.
        assert_eq!(
            RailHeading::Forward.flipped().flipped(),
            RailHeading::Forward
        );
        assert_eq!(
            RailHeading::Forward.flipped().sign(),
            -RailHeading::Forward.sign()
        );
    }

    #[test]
    fn sign_and_facing_offset_match_heading() {
        assert_eq!(RailHeading::Forward.sign(), 1.0);
        assert_eq!(RailHeading::Backward.sign(), -1.0);
        assert_eq!(RailHeading::Forward.facing_offset(), 0.0);
        assert_eq!(RailHeading::Backward.facing_offset(), PI);
    }
}
