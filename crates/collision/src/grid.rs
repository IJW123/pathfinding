use bevy::prelude::*;

use crate::aabb::Aabb;
use crate::constants::CELL_SIZE;

/// Cell containing `point`. Floor-divide so negative coordinates bucket correctly.
#[must_use]
pub fn cell_of(point: Vec2) -> IVec2 {
    IVec2::new(
        (point.x / CELL_SIZE).floor() as i32,
        (point.y / CELL_SIZE).floor() as i32,
    )
}

/// All cells an AABB touches, inclusive of its corner cells.
pub fn cells_for_aabb(aabb: &Aabb) -> impl Iterator<Item = IVec2> {
    let min = cell_of(aabb.min);
    let max = cell_of(aabb.max);
    (min.y..=max.y).flat_map(move |y| (min.x..=max.x).map(move |x| IVec2::new(x, y)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cell_of_negative_coords_floor() {
        assert_eq!(cell_of(Vec2::new(-1.0, -1.0)), IVec2::new(-1, -1));
        assert_eq!(cell_of(Vec2::new(1.0, 1.0)), IVec2::ZERO);
    }

    #[test]
    fn cells_for_aabb_spans_inclusive() {
        // 80u cells; an AABB from (-10,-10) to (90,10) touches cells x∈{-1,0,1}, y∈{-1,0}.
        let aabb = Aabb {
            min: Vec2::new(-10.0, -10.0),
            max: Vec2::new(90.0, 10.0),
        };
        let cells: Vec<IVec2> = cells_for_aabb(&aabb).collect();
        assert_eq!(cells.len(), 6);
        assert!(cells.contains(&IVec2::new(-1, -1)));
        assert!(cells.contains(&IVec2::new(1, 0)));
    }
}
