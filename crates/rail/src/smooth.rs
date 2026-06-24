use bevy::math::Vec2;

use crate::constants::{MAX_SAMPLES_PER_SPAN, MIN_SAMPLES_PER_SPAN};

/// Centripetal exponent (α = 0.5). Centripetal Catmull-Rom avoids the cusps and self-intersections
/// uniform/chordal variants produce at sharp corners — exactly the corners a rail must round.
const ALPHA: f32 = 0.5;

/// Below this knot spacing two control points are treated as coincident and the term collapses to a
/// constant, dodging a divide-by-zero. Reflection keeps endpoint knots non-zero, so this only bites
/// on duplicate authored points.
const KNOT_EPSILON: f32 = 1e-6;

/// Round a coarse authored polyline into a dense smooth one by fitting a centripetal Catmull-Rom
/// spline through the waypoints and flattening it. Each span is subdivided so its segments are about
/// `target_segment_len` long — resolution follows distance, not a fixed count, so a long span and a
/// short one end up with comparably-sized segments instead of the long one going blocky. Per-span
/// count is clamped to [`MIN_SAMPLES_PER_SPAN`]..=[`MAX_SAMPLES_PER_SPAN`] (round short corners,
/// cap runaway long spans).
///
/// The curve **interpolates** the waypoints (it passes through every authored point), and the first
/// and last authored points are preserved exactly — endpoint tangents come from a *reflected*
/// phantom neighbor (`2*p0 - p1`), not a duplicated one, so the endpoint knot intervals stay
/// non-zero and the dead-stop ends land precisely where authored.
///
/// Fewer than three points can't be rounded (a single segment or less), so the input is returned
/// unchanged. Every track producer — today's RON, tomorrow's pathfinder — funnels through here.
#[must_use]
pub fn smooth_track(authored: &[Vec2], target_segment_len: f32) -> Vec<Vec2> {
    if authored.len() < 3 || target_segment_len <= 0.0 {
        return authored.to_vec();
    }

    let n = authored.len();
    let mut out = Vec::new();
    out.push(authored[0]);
    for i in 0..n - 1 {
        // Reflected phantoms at the ends keep knot spacing non-zero and the endpoint tangent natural.
        let p0 = if i == 0 {
            2.0 * authored[0] - authored[1]
        } else {
            authored[i - 1]
        };
        let p1 = authored[i];
        let p2 = authored[i + 1];
        let p3 = if i + 2 >= n {
            2.0 * authored[n - 1] - authored[n - 2]
        } else {
            authored[i + 2]
        };
        // Subdivide this span by its chord length — a cheap proxy for the (slightly longer) arc, so
        // segments land at or just under the target.
        let samples = (p1.distance(p2) / target_segment_len).ceil() as usize;
        let samples = samples.clamp(MIN_SAMPLES_PER_SPAN, MAX_SAMPLES_PER_SPAN);
        for step in 1..=samples {
            let u = step as f32 / samples as f32;
            out.push(catmull_rom_centripetal(p0, p1, p2, p3, u));
        }
    }
    out
}

/// One point on the centripetal Catmull-Rom segment between `p1` and `p2`, with `u` in `[0, 1]`
/// mapped across that span's knot interval. `p0`/`p3` are the outer control points (Barry-Goldman
/// pyramidal form).
fn catmull_rom_centripetal(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, u: f32) -> Vec2 {
    let t0 = 0.0;
    let t1 = t0 + p0.distance(p1).powf(ALPHA);
    let t2 = t1 + p1.distance(p2).powf(ALPHA);
    let t3 = t2 + p2.distance(p3).powf(ALPHA);
    let t = t1 + u * (t2 - t1);

    let a1 = knot_lerp(p0, p1, t0, t1, t);
    let a2 = knot_lerp(p1, p2, t1, t2, t);
    let a3 = knot_lerp(p2, p3, t2, t3, t);
    let b1 = knot_lerp(a1, a2, t0, t2, t);
    let b2 = knot_lerp(a2, a3, t1, t3, t);
    knot_lerp(b1, b2, t1, t2, t)
}

/// Linear interpolation in knot space: `pa` at `ta`, `pb` at `tb`, evaluated at `t`. A zero-width
/// interval (coincident knots) collapses to `pa` instead of dividing by zero.
fn knot_lerp(pa: Vec2, pb: Vec2, ta: f32, tb: f32, t: f32) -> Vec2 {
    let span = tb - ta;
    if span.abs() < KNOT_EPSILON {
        pa
    } else {
        pa * ((tb - t) / span) + pb * ((t - ta) / span)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passes_through_authored_waypoints() {
        let authored = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(100.0, 0.0),
            Vec2::new(100.0, 100.0),
            Vec2::new(0.0, 100.0),
        ];
        // 100-unit spans at a 10-unit target ⇒ ~10 samples each, comfortably above the floor.
        let smoothed = smooth_track(&authored, 10.0);
        for point in &authored {
            let hit = smoothed.iter().any(|s| s.distance(*point) < 1e-3);
            assert!(hit, "smoothed curve misses authored waypoint {point:?}");
        }
    }

    #[test]
    fn endpoints_preserved_exactly() {
        let authored = vec![
            Vec2::new(-5.0, 3.0),
            Vec2::new(40.0, 10.0),
            Vec2::new(80.0, -20.0),
        ];
        let smoothed = smooth_track(&authored, 10.0);
        assert_eq!(*smoothed.first().expect("non-empty"), authored[0]);
        assert_eq!(
            *smoothed.last().expect("non-empty"),
            *authored.last().unwrap()
        );
    }

    #[test]
    fn corner_turns_continuously() {
        // A right-angle L: the raw polyline turns 90° in a single vertex. After smoothing, the
        // per-segment heading change must be bounded (no discontinuous snap), proving the corner
        // actually rounds rather than staying sharp.
        let authored = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(100.0, 0.0),
            Vec2::new(100.0, 100.0),
        ];
        // 100-unit spans at a 5-unit target ⇒ ~20 samples each (≥ the old fixed 16), so the corner is
        // sampled at least as finely as before and the per-segment turn stays bounded.
        let smoothed = smooth_track(&authored, 5.0);
        let mut max_turn: f32 = 0.0;
        for w in smoothed.windows(3) {
            let d0 = (w[1] - w[0]).normalize_or_zero();
            let d1 = (w[2] - w[1]).normalize_or_zero();
            let turn = d0.angle_to(d1).abs();
            max_turn = max_turn.max(turn);
        }
        // A sharp corner would show a ~90° (1.57 rad) jump at one vertex; rounded stays well under.
        assert!(
            max_turn < 0.5,
            "corner did not round: max per-segment turn {max_turn} rad"
        );
    }

    #[test]
    fn short_input_returned_unchanged() {
        let two = vec![Vec2::ZERO, Vec2::new(10.0, 0.0)];
        assert_eq!(smooth_track(&two, 8.0), two);
    }
}
