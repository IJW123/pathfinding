use std::cmp::Ordering;

use bevy::prelude::*;

use crate::constants::{MIN_SEGMENT_LEN, TARGET_SEGMENT_LEN};
use crate::smooth::smooth_track;

/// World pose of a point on the track: position plus the tangent angle (radians, z-axis) pointing
/// along *increasing* arc-length. Consumers facing the other way add π themselves.
#[derive(Debug, Clone, Copy)]
pub struct RailPose {
    pub position: Vec2,
    pub angle: f32,
}

/// A finite rail track: the smoothed centerline polyline plus the cumulative arc-length at each
/// vertex, so a scalar arc-length maps to a world pose in one lookup. Immutable once built —
/// geometry is authored/generated up front, never edited at runtime.
#[derive(Component)]
pub struct RailTrack {
    /// Smoothed centerline. `cumulative[i]` is the arc-length from the start to `points[i]`.
    points: Vec<Vec2>,
    /// Same length as `points`; `cumulative[0] == 0`, monotonically increasing.
    cumulative: Vec<f32>,
}

impl RailTrack {
    /// Build a track from authored waypoints: round the corners ([`smooth_track`]) then precompute
    /// cumulative arc-lengths. This is the single seam every producer goes through — RON today, a
    /// pathfinder later — so it owns the smoothing and length bookkeeping once.
    #[must_use]
    pub fn new(authored: Vec<Vec2>) -> Self {
        debug_assert!(
            authored.len() >= 2,
            "rail track needs at least 2 authored points"
        );
        let smoothed = smooth_track(&authored, TARGET_SEGMENT_LEN);

        // Collapse consecutive coincident points (e.g. from a duplicate authored waypoint) so no
        // retained segment is zero-length — otherwise `sample`'s tangent would `atan2(0, 0)` and snap
        // facing to +x at that vertex.
        let mut points: Vec<Vec2> = Vec::with_capacity(smoothed.len());
        for p in smoothed {
            if points
                .last()
                .is_none_or(|last| last.distance(p) > MIN_SEGMENT_LEN)
            {
                points.push(p);
            }
        }

        let mut cumulative = Vec::with_capacity(points.len());
        cumulative.push(0.0);
        let mut acc = 0.0;
        for w in points.windows(2) {
            acc += w[0].distance(w[1]);
            cumulative.push(acc);
        }
        debug_assert!(acc > 0.0, "rail track has zero total length");

        Self { points, cumulative }
    }

    /// Total track length in world units (== arc-length at the far end).
    #[must_use]
    pub fn length(&self) -> f32 {
        self.cumulative.last().copied().unwrap_or(0.0)
    }

    /// The smoothed centerline vertices, for the renderer.
    #[must_use]
    pub fn points(&self) -> &[Vec2] {
        &self.points
    }

    /// World pose at arc-length `s`, clamped to `[0, length]` (dead-stop ends). Finds the containing
    /// segment, lerps position, and reads the tangent angle off that segment.
    #[must_use]
    pub fn sample(&self, s: f32) -> RailPose {
        // A producer that emitted fewer than 2 distinct points leaves nothing to interpolate.
        // `new`'s debug_assert flags this in dev; in release we pin to the sole point (facing +x)
        // rather than index `points[i + 1]` out of bounds. This is the seam's release safety net for
        // a future pathfinder that hands over a degenerate path.
        if self.points.len() < 2 {
            return RailPose {
                position: self.points.first().copied().unwrap_or(Vec2::ZERO),
                angle: 0.0,
            };
        }
        let s = s.clamp(0.0, self.length());
        let last_seg = self.points.len().saturating_sub(2);
        let i = match self
            .cumulative
            .binary_search_by(|c| c.partial_cmp(&s).unwrap_or(Ordering::Equal))
        {
            Ok(idx) => idx.min(last_seg),
            Err(idx) => idx.saturating_sub(1).min(last_seg),
        };

        let seg_start = self.cumulative[i];
        let seg_len = self.cumulative[i + 1] - seg_start;
        let t = if seg_len > 0.0 {
            (s - seg_start) / seg_len
        } else {
            0.0
        };

        let a = self.points[i];
        let b = self.points[i + 1];
        let dir = b - a;
        RailPose {
            position: a.lerp(b, t),
            angle: dir.y.atan2(dir.x),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::FRAC_PI_2;

    /// A straight horizontal track avoids smoothing artifacts (collinear points stay collinear), so
    /// arc-length math is exact and assertable.
    fn straight() -> RailTrack {
        RailTrack::new(vec![Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0)])
    }

    #[test]
    fn endpoints_and_length() {
        let track = straight();
        assert!((track.length() - 100.0).abs() < 1e-3);
        assert!(track.sample(0.0).position.distance(Vec2::ZERO) < 1e-3);
        assert!(
            track
                .sample(track.length())
                .position
                .distance(Vec2::new(100.0, 0.0))
                < 1e-3
        );
    }

    #[test]
    fn midpoint_and_tangent() {
        let track = straight();
        let mid = track.sample(50.0);
        assert!(mid.position.distance(Vec2::new(50.0, 0.0)) < 1e-3);
        assert!(mid.angle.abs() < 1e-3, "tangent points +x along the track");
    }

    #[test]
    fn clamps_past_both_ends() {
        let track = straight();
        assert!(track.sample(-50.0).position.distance(Vec2::ZERO) < 1e-3);
        assert!(
            track
                .sample(1_000.0)
                .position
                .distance(Vec2::new(100.0, 0.0))
                < 1e-3
        );
    }

    #[test]
    fn degenerate_track_samples_without_panic() {
        // A producer emitting a single point (or none) must not panic `sample` in release. Build the
        // struct directly to bypass `new`'s dev-only debug_assert and exercise the release guard.
        let single = RailTrack {
            points: vec![Vec2::new(7.0, 3.0)],
            cumulative: vec![0.0],
        };
        assert!(single.sample(50.0).position.distance(Vec2::new(7.0, 3.0)) < 1e-3);

        let empty = RailTrack {
            points: vec![],
            cumulative: vec![0.0],
        };
        assert_eq!(empty.sample(0.0).position, Vec2::ZERO);
    }

    #[test]
    fn vertical_track_tangent() {
        let track = RailTrack::new(vec![Vec2::new(0.0, 0.0), Vec2::new(0.0, 80.0)]);
        let mid = track.sample(40.0);
        assert!(mid.position.distance(Vec2::new(0.0, 40.0)) < 1e-3);
        assert!(
            (mid.angle - FRAC_PI_2).abs() < 1e-3,
            "tangent points +y, got {}",
            mid.angle
        );
    }

    #[test]
    fn l_shape_has_length_beyond_chord() {
        // Rounded L: arc-length exceeds the straight diagonal chord (it bends around the corner),
        // yet stays bounded near the authored box — a Catmull-Rom spline overshoots its control hull
        // only modestly, never flying off, so a generous margin (not a tight box) is the right check.
        let track = RailTrack::new(vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(100.0, 0.0),
            Vec2::new(100.0, 100.0),
        ]);
        assert!(track.length() > Vec2::new(100.0, 100.0).length());
        const MARGIN: f32 = 20.0;
        for s in [0.0, 25.0, 90.0, track.length()] {
            let p = track.sample(s).position;
            assert!(
                (-MARGIN..=100.0 + MARGIN).contains(&p.x)
                    && (-MARGIN..=100.0 + MARGIN).contains(&p.y),
                "sample {p:?} diverged far past the authored bounds"
            );
        }
    }
}
