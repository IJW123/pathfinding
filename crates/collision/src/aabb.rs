use bevy::prelude::*;

/// Axis-aligned bounding box in world space. Closed intervals: edge-touching boxes overlap.
#[derive(Clone, Copy, Debug)]
pub struct Aabb {
    pub min: Vec2,
    pub max: Vec2,
}

impl Aabb {
    #[must_use]
    pub fn overlaps(&self, other: &Aabb) -> bool {
        self.min.x <= other.max.x
            && other.min.x <= self.max.x
            && self.min.y <= other.max.y
            && other.min.y <= self.max.y
    }

    #[must_use]
    pub fn inflated(&self, margin: f32) -> Aabb {
        Aabb {
            min: self.min - Vec2::splat(margin),
            max: self.max + Vec2::splat(margin),
        }
    }

    #[must_use]
    pub fn translated(&self, delta: Vec2) -> Aabb {
        Aabb {
            min: self.min + delta,
            max: self.max + delta,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_at(center: Vec2) -> Aabb {
        Aabb {
            min: center - Vec2::ONE,
            max: center + Vec2::ONE,
        }
    }

    #[test]
    fn overlaps_disjoint_touching_overlapping() {
        let a = unit_at(Vec2::ZERO);
        assert!(!a.overlaps(&unit_at(Vec2::new(3.0, 0.0))));
        assert!(a.overlaps(&unit_at(Vec2::new(2.0, 0.0)))); // edge touch counts
        assert!(a.overlaps(&unit_at(Vec2::new(1.0, 1.0))));
    }

    #[test]
    fn inflated_grows_both_sides() {
        let a = unit_at(Vec2::ZERO).inflated(0.5);
        assert_eq!(a.min, Vec2::splat(-1.5));
        assert_eq!(a.max, Vec2::splat(1.5));
    }

    #[test]
    fn translated_moves_both_corners() {
        let a = unit_at(Vec2::ZERO).translated(Vec2::new(2.0, -1.0));
        assert_eq!(a.min, Vec2::new(1.0, -2.0));
        assert_eq!(a.max, Vec2::new(3.0, 0.0));
    }
}
