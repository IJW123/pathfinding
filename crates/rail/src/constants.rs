use bevy::input::keyboard::KeyCode;
use bevy::math::Vec2;

/// Along-track speed (world units/sec) while the drive key is held. Same ballpark as the carrier's
/// `CONTROL_SPEED` so the loco doesn't feel out of place next to free-moving units.
pub const LOCO_SPEED: f32 = 300.0;

/// Half-extents of the locomotive's body: longer along its travel axis than across, so it reads as
/// a train car. Used for both the click-picking collider and the rendered sprite.
pub const LOCO_HALF_EXTENTS: Vec2 = Vec2::new(45.0, 18.0);

/// Z layer for the locomotive: alongside the player (above obstacles/walls/track), so it draws on
/// top of everything it travels over. Mirrors `PLAYER_Z` in `player/constants.rs`.
pub const LOCO_Z: f32 = 1.0;

/// Z layer for the rail track: above contour lines (0.1) and walls (0.0), below obstacles (0.5) so
/// obstacles still occlude it. A world-side ordering policy intrinsic to the kind, like `PLAYER_Z`.
pub const TRACK_Z: f32 = 0.2;

/// Target length (world units) of each smoothed line segment. Each authored span is subdivided by
/// its own chord length, so segment size stays roughly constant across the map however far apart the
/// waypoints sit — far-apart waypoints no longer go blocky. See [`crate::smooth::smooth_track`].
pub const TARGET_SEGMENT_LEN: f32 = 20.0;

/// Floor on samples per authored span. Without it a span shorter than [`TARGET_SEGMENT_LEN`] would
/// get a single sample (the authored point verbatim), leaving that corner sharp — so this guarantees
/// even short corners still round.
pub const MIN_SAMPLES_PER_SPAN: usize = 4;

/// Ceiling on samples per authored span: a runaway guard so one very long span can't emit thousands
/// of vertices.
pub const MAX_SAMPLES_PER_SPAN: usize = 64;

/// Consecutive smoothed points closer than this (world units) are collapsed when building a
/// [`RailTrack`](crate::track::RailTrack), so every retained segment has non-zero length and the
/// tangent (`atan2(dy, dx)`) is always well-defined.
pub const MIN_SEGMENT_LEN: f32 = 1e-3;

/// Drives the selected locomotive along the track in its current heading while held.
pub const DRIVE_KEY: KeyCode = KeyCode::ArrowUp;

/// Flips the selected locomotive's heading 180° on press (the "turning button").
pub const TURN_KEY: KeyCode = KeyCode::KeyT;
