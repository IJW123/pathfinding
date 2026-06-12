use std::error::Error;
use std::fmt::{self, Display, Formatter};

use bevy::prelude::*;

use crate::constants::HULL_COLLINEAR_EPSILON;

/// A validated convex polygon: at least 3 points, convex, CCW winding, non-degenerate.
/// The private field means downstream code (SAT, broad phase, render) never re-checks —
/// the type carries the proof.
#[derive(Debug)]
pub struct ConvexHull {
    points: Vec<Vec2>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum HullError {
    TooFewPoints,
    NotConvex,
    Degenerate,
}

impl Display for HullError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooFewPoints => write!(f, "convex hull needs at least 3 points"),
            Self::NotConvex => write!(f, "points do not form a convex polygon"),
            Self::Degenerate => write!(f, "points are collinear (zero-area hull)"),
        }
    }
}

impl Error for HullError {}

impl ConvexHull {
    /// Validates the points and normalizes winding to CCW (a CW hull is reversed — lossless,
    /// same polygon). Convexity is judged per corner by the sine of the turn angle
    /// (`cross / (|ab|·|bc|)`), which is scale-free; collinear corners and duplicate
    /// consecutive points contribute nothing.
    ///
    /// # Errors
    /// `TooFewPoints` for fewer than 3 points, `NotConvex` if consecutive turns disagree in
    /// direction, `Degenerate` if every corner is collinear (zero-area hull).
    pub fn new(mut points: Vec<Vec2>) -> Result<Self, HullError> {
        if points.len() < 3 {
            return Err(HullError::TooFewPoints);
        }
        let sign = turn_sign(&points)?;
        if sign < 0.0 {
            points.reverse();
        }
        Ok(Self { points })
    }

    #[must_use]
    pub fn points(&self) -> &[Vec2] {
        &self.points
    }
}

/// Common turn direction of all corners: `1.0` for CCW, `-1.0` for CW.
///
/// # Errors
/// `NotConvex` on mixed turn directions, `Degenerate` if no corner turns at all.
fn turn_sign(points: &[Vec2]) -> Result<f32, HullError> {
    let n = points.len();
    let mut sign = 0.0_f32;
    for (i, &a) in points.iter().enumerate() {
        let b = points[(i + 1) % n];
        let c = points[(i + 2) % n];
        let denom = (b - a).length() * (c - b).length();
        if denom <= 0.0 {
            continue;
        }
        let turn = (b - a).perp_dot(c - b) / denom;
        if turn.abs() <= HULL_COLLINEAR_EPSILON {
            continue;
        }
        if sign != 0.0 && turn.signum() != sign {
            return Err(HullError::NotConvex);
        }
        sign = turn.signum();
    }
    if sign == 0.0 {
        return Err(HullError::Degenerate);
    }
    Ok(sign)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ccw_square() -> Vec<Vec2> {
        vec![
            Vec2::new(-1.0, -1.0),
            Vec2::new(1.0, -1.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(-1.0, 1.0),
        ]
    }

    #[test]
    fn too_few_points_rejected() {
        let points = vec![Vec2::ZERO, Vec2::X];
        assert_eq!(
            ConvexHull::new(points).unwrap_err(),
            HullError::TooFewPoints
        );
    }

    #[test]
    fn concave_rejected() {
        // Arrowhead: the (0.2, 0.0) notch turns the other way.
        let points = vec![
            Vec2::new(-1.0, -1.0),
            Vec2::new(1.0, -1.0),
            Vec2::new(0.2, 0.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(-1.0, 1.0),
        ];
        assert_eq!(ConvexHull::new(points).unwrap_err(), HullError::NotConvex);
    }

    #[test]
    fn collinear_rejected() {
        let points = vec![Vec2::ZERO, Vec2::X, Vec2::new(2.0, 0.0)];
        assert_eq!(ConvexHull::new(points).unwrap_err(), HullError::Degenerate);
    }

    #[test]
    fn ccw_input_unchanged() {
        let points = ccw_square();
        let hull = ConvexHull::new(points.clone()).expect("valid hull");
        assert_eq!(hull.points(), points.as_slice());
    }

    #[test]
    fn cw_input_reversed_to_ccw() {
        let mut points = ccw_square();
        points.reverse();
        let hull = ConvexHull::new(points).expect("valid hull");
        assert_eq!(hull.points(), ccw_square().as_slice());
    }

    #[test]
    fn duplicate_consecutive_points_tolerated() {
        let mut points = ccw_square();
        points.insert(1, points[1]);
        let hull = ConvexHull::new(points).expect("valid hull");
        assert_eq!(hull.points().len(), 5);
    }

    #[test]
    fn tiny_hull_scale_free() {
        // 1e-3-sized triangle: raw cross products (~1e-6) sit far below any absolute
        // epsilon, but the sine-of-turn check is scale-free.
        let points = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(1e-3, 0.0),
            Vec2::new(0.0, 1e-3),
        ];
        assert!(ConvexHull::new(points).is_ok());
    }
}
