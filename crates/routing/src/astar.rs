use bevy::math::Vec2;
use pathfinding::directed::astar::astar;

use crate::constants::{COST_SCALE, MIN_ENDPOINT_SEP, NEIGHBOURS, SIMPLIFY_TOLERANCE_STEPS};
use crate::grid::Grid;
use crate::profile::PathProfile;
use crate::sampler::ElevationSampler;
use crate::simplify::simplify;

/// Route from `start` to `goal` over `field`, staying under `profile.max_grade` on every step.
///
/// Returns the connecting polyline in world space (exact `start`/`goal` at the ends), thinned for a
/// spline smoother. Returns `None` when no grade-legal route exists, or when the endpoints are
/// closer than [`MIN_ENDPOINT_SEP`] (nothing to connect).
///
/// Endpoints outside `field.bounds()` snap to the nearest in-bounds node for the search; the exact
/// out-of-map `start`/`goal` are still placed at the ends of the result.
///
/// # Panics
/// Debug builds assert `profile.max_grade > 0.0`; the cost divides by it and a non-positive cutoff
/// would forbid every non-flat edge.
#[must_use]
pub fn find_path(
    field: &impl ElevationSampler,
    start: Vec2,
    goal: Vec2,
    profile: &PathProfile,
) -> Option<Vec<Vec2>> {
    debug_assert!(profile.max_grade > 0.0, "max_grade must be > 0");

    if start.distance(goal) < MIN_ENDPOINT_SEP {
        return None;
    }

    let grid = Grid::new(field.bounds(), profile.step);
    let start_node = grid.to_node(start);
    let goal_node = grid.to_node(goal);

    // Within a single cell there is nothing for A* to search: connect the endpoints directly.
    if start_node == goal_node {
        return Some(vec![start, goal]);
    }

    let goal_world = grid.to_world(goal_node);
    let cost_to = |here: Vec2, there: Vec2| -> Option<u64> {
        let horiz = here.distance(there);
        let grade = (field.height(there) - field.height(here)).abs() / horiz;
        (grade <= profile.max_grade).then(|| {
            let factor = 1.0 + profile.grade_cost_weight * grade / profile.max_grade;
            (horiz * factor * COST_SCALE).round() as u64
        })
    };
    let cost_to = &cost_to;

    let (nodes, _cost) = astar(
        &start_node,
        move |&node| {
            let here = grid.to_world(node);
            NEIGHBOURS.into_iter().filter_map(move |off| {
                let nb = node + off;
                grid.in_bounds(nb)
                    .then(|| cost_to(here, grid.to_world(nb)).map(|c| (nb, c)))
                    .flatten()
            })
        },
        move |&node| (grid.to_world(node).distance(goal_world) * COST_SCALE).round() as u64,
        |&node| node == goal_node,
    )?;

    let mut world: Vec<Vec2> = nodes.into_iter().map(|n| grid.to_world(n)).collect();
    // Anchor the polyline to the caller's exact endpoints (node centers are off by up to half a
    // cell, and an out-of-bounds endpoint snapped during the search).
    world[0] = start;
    *world.last_mut().expect("astar path has >= 2 nodes") = goal;

    Some(simplify(&world, profile.step * SIMPLIFY_TOLERANCE_STEPS))
}
