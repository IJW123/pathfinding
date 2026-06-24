//! End-to-end `find_path` behaviour against hand-built mock terrain. The mocks implement
//! `ElevationSampler` directly, so these tests stay free of any heightmap/game crate.

use bevy::math::{Rect, Vec2};
use routing::{ElevationSampler, PathProfile, find_path};

/// Square field, world `[0, size] x [0, size]`, height = `f(x, y)`. Bounds clamp queries to the edge.
struct MockField {
    size: f32,
    height_fn: fn(Vec2) -> f32,
}

impl MockField {
    fn new(size: f32, height_fn: fn(Vec2) -> f32) -> Self {
        Self { size, height_fn }
    }
}

impl ElevationSampler for MockField {
    fn height(&self, p: Vec2) -> f32 {
        let clamped = p.clamp(Vec2::ZERO, Vec2::splat(self.size));
        (self.height_fn)(clamped)
    }

    fn bounds(&self) -> Rect {
        Rect::from_corners(Vec2::ZERO, Vec2::splat(self.size))
    }
}

fn profile(max_grade: f32) -> PathProfile {
    PathProfile {
        max_grade,
        grade_cost_weight: 1.0,
        step: 10.0,
    }
}

/// Sum of segment lengths — a route's total run.
fn path_length(path: &[Vec2]) -> f32 {
    path.windows(2).map(|w| w[0].distance(w[1])).sum()
}

#[test]
fn flat_field_routes_near_straight() {
    let field = MockField::new(200.0, |_| 0.0);
    let start = Vec2::new(10.0, 100.0);
    let goal = Vec2::new(190.0, 100.0);
    let path =
        find_path(&field, start, goal, &profile(0.05)).expect("flat field is always routable");

    assert_eq!(path.first().copied(), Some(start));
    assert_eq!(path.last().copied(), Some(goal));
    // No terrain to avoid ⇒ the route should be the straight run, give or take grid snapping.
    let straight = start.distance(goal);
    assert!(
        path_length(&path) < straight * 1.05,
        "flat route {} much longer than straight {straight}",
        path_length(&path),
    );
}

#[test]
fn gentle_ramp_under_cutoff_is_crossed() {
    // Height ramps 0.02 per world unit in x. A horizontal step has grade 0.02 < 0.05 cutoff.
    let field = MockField::new(200.0, |p| p.x * 0.02);
    let start = Vec2::new(10.0, 100.0);
    let goal = Vec2::new(190.0, 100.0);
    let path =
        find_path(&field, start, goal, &profile(0.05)).expect("0.02 grade is under the cutoff");

    // It can climb straight up the ramp, so the route stays near-straight rather than detouring.
    let straight = start.distance(goal);
    assert!(path_length(&path) < straight * 1.2);
}

#[test]
fn wall_with_a_gap_forces_a_detour() {
    // A cliff-walled band at x in [90, 110], impassably tall except for a gap at y >= 150. Crossing
    // anywhere below the gap has grade 100 (1000 / 10) >> cutoff, so the route must climb north to
    // the gap and back — a long, unmistakable detour rather than the straight east run.
    let field = MockField::new(200.0, |p| {
        let in_band = (90.0..=110.0).contains(&p.x);
        if in_band && p.y < 150.0 { 1000.0 } else { 0.0 }
    });
    let start = Vec2::new(10.0, 10.0);
    let goal = Vec2::new(190.0, 10.0);
    let path = find_path(&field, start, goal, &profile(0.05)).expect("the gap makes a route exist");

    let straight = start.distance(goal);
    assert!(
        path_length(&path) > straight * 1.3,
        "expected a long detour, got {} vs straight {straight}",
        path_length(&path),
    );
    let reaches_gap = path.iter().any(|p| p.y >= 140.0);
    assert!(reaches_gap, "route should climb to the gap at y >= 150");
}

#[test]
fn impassable_wall_returns_none() {
    // A full-height cliff spanning the entire y-extent at x≈100: every east step across it has grade
    // 1000/10 = 100, far over any cutoff, and there is no gap to route through.
    let field = MockField::new(200.0, |p| if p.x > 100.0 { 1000.0 } else { 0.0 });
    let start = Vec2::new(10.0, 100.0);
    let goal = Vec2::new(190.0, 100.0);
    assert!(find_path(&field, start, goal, &profile(0.05)).is_none());
}

#[test]
fn coincident_endpoints_return_none() {
    let field = MockField::new(200.0, |_| 0.0);
    let p = Vec2::new(50.0, 50.0);
    assert!(find_path(&field, p, p, &profile(0.05)).is_none());
}

#[test]
fn endpoints_within_one_cell_connect_directly() {
    let field = MockField::new(200.0, |_| 0.0);
    let start = Vec2::new(50.0, 50.0);
    let goal = Vec2::new(53.0, 51.0); // same node, but distinct points
    let path = find_path(&field, start, goal, &profile(0.05)).expect("distinct points connect");
    assert_eq!(path, vec![start, goal]);
}
