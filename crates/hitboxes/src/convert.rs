use bevy::prelude::*;
use parry2d::math::{Pose, Vector};

/// Bevy and parry ship different glam majors (0.30 vs 0.33), so their `Vec2`s are distinct
/// types. These field-wise converts are the single seam between the two math worlds.
#[must_use]
pub fn vec2_to_parry(v: Vec2) -> Vector {
    Vector::new(v.x, v.y)
}

#[must_use]
pub fn parry_to_vec2(v: Vector) -> Vec2 {
    Vec2::new(v.x, v.y)
}

/// 2D pose (isometry) of a Bevy transform. The angle comes from the rotated X axis rather than
/// Euler extraction, so any Z-rotation quaternion works without singularities.
#[must_use]
pub fn transform_to_pose(transform: &Transform) -> Pose {
    let dir = transform.rotation * Vec3::X;
    Pose::new(
        vec2_to_parry(transform.translation.truncate()),
        dir.y.atan2(dir.x),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::FRAC_PI_2;

    #[test]
    fn pose_rotates_then_translates() {
        // 90° CCW + translate (10, 5): local (1, 0) lands on (10, 6).
        let tx =
            Transform::from_xyz(10.0, 5.0, 0.0).with_rotation(Quat::from_rotation_z(FRAC_PI_2));
        let pose = transform_to_pose(&tx);
        let p = pose.transform_point(Vector::new(1.0, 0.0));
        assert!((parry_to_vec2(p) - Vec2::new(10.0, 6.0)).length() < 1e-4);
    }

    #[test]
    fn vec2_roundtrip() {
        let v = Vec2::new(-3.5, 7.25);
        assert_eq!(parry_to_vec2(vec2_to_parry(v)), v);
    }
}
