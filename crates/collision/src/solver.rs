use bevy::prelude::*;

use crate::broad_phase::{CandidatePair, CollisionPairs, DynamicBodies, DynamicBody, PairBodies};
use crate::constants::{
    COLLISION_EPSILON, PENETRATION_PERCENT, PENETRATION_SLOP, SOLVER_ITERATIONS,
};
use crate::events::CollisionEvent;
use crate::manifold::Manifold;
use crate::narrow_phase::test_world_pair;
use crate::static_index::StaticColliderIndex;

/// Share of a correction each side absorbs (both non-negative; a moves against the a→b normal,
/// b along it). Mass slots in here later — weights become inverse-mass ratios; today statics
/// are immovable and dynamics split equally.
pub struct PairWeights {
    pub a: f32,
    pub b: f32,
}

#[must_use]
pub fn pair_weights(a_static: bool, b_static: bool) -> PairWeights {
    match (a_static, b_static) {
        (true, true) => PairWeights { a: 0.0, b: 0.0 },
        (true, false) => PairWeights { a: 0.0, b: 1.0 },
        (false, true) => PairWeights { a: 1.0, b: 0.0 },
        (false, false) => PairWeights { a: 0.5, b: 0.5 },
    }
}

/// A pass-0 (pre-resolution) contact, kept for event emission. Normal points a→b.
pub struct InitialContact {
    pub a: Entity,
    pub b: Entity,
    pub manifold: Manifold,
}

pub struct SolveOutcome {
    pub iterations_run: usize,
}

/// Move a body and keep its cached world geometry exact (corrections are pure translations).
fn shift(body: &mut DynamicBody, delta: Vec2) {
    body.offset += delta;
    body.shape.translate(delta);
    body.aabb = body.aabb.translated(delta);
}

/// Correction for one contact: resolve penetration beyond the slop, damped by
/// `PENETRATION_PERCENT`. Returns false when the contact is already within slop.
fn apply_correction(bodies: &mut [DynamicBody], pair: &CandidatePair, manifold: &Manifold) -> bool {
    let push_len = (manifold.depth - PENETRATION_SLOP).max(0.0) * PENETRATION_PERCENT;
    if push_len <= 0.0 {
        return false;
    }
    let push = manifold.normal * push_len;
    match pair.bodies {
        PairBodies::DynamicStatic { body, .. } => {
            // Static side never moves; the dynamic `a` takes the full correction against the normal.
            let weights = pair_weights(false, true);
            shift(&mut bodies[body], -push * weights.a);
        }
        PairBodies::DynamicDynamic { a, b } => {
            let weights = pair_weights(false, false);
            let (left, right) = bodies.split_at_mut(b);
            shift(&mut left[a], -push * weights.a);
            shift(&mut right[0], push * weights.b);
        }
    }
    true
}

/// Manifold for a candidate pair from current cached geometry. Normal points a→b
/// (dynamic→static for mixed pairs, lower→higher body index otherwise). The cached AABBs
/// (kept exact by `shift`) gate the SAT: separated near-pairs cost 4 comparisons per pass,
/// not a full SAT — and the re-check each pass keeps up with mid-solve motion both ways.
fn pair_manifold(
    bodies: &[DynamicBody],
    statics: &StaticColliderIndex,
    pair: &CandidatePair,
) -> Option<Manifold> {
    match pair.bodies {
        PairBodies::DynamicStatic { body, anchor } => {
            let entry = &statics.entries[anchor];
            bodies[body]
                .aabb
                .overlaps(&entry.aabb)
                .then(|| test_world_pair(&bodies[body].shape, &entry.shape))
                .flatten()
        }
        PairBodies::DynamicDynamic { a, b } => bodies[a]
            .aabb
            .overlaps(&bodies[b].aabb)
            .then(|| test_world_pair(&bodies[a].shape, &bodies[b].shape))
            .flatten(),
    }
}

/// Gauss-Seidel positional solver. Pass 0 tests every candidate pair, records pre-resolution
/// contacts (event emission preserves today's semantics: any touching collider pair), and
/// corrects solid pairs immediately. Passes 1.. re-test only solid pairs against *current*
/// positions — corrections propagate through chains one link per pass — and stop early once a
/// pass applies nothing. Corrections accumulate in `DynamicBody::offset`; Transforms are
/// untouched here.
pub fn solve_pairs(
    bodies: &mut [DynamicBody],
    statics: &StaticColliderIndex,
    pairs: &[CandidatePair],
    contacts: &mut Vec<InitialContact>,
) -> SolveOutcome {
    let mut applied = false;
    for pair in pairs {
        let Some(manifold) = pair_manifold(bodies, statics, pair) else {
            continue;
        };
        let (a, b) = match pair.bodies {
            PairBodies::DynamicStatic { body, anchor } => {
                (bodies[body].entity, statics.entries[anchor].entity)
            }
            PairBodies::DynamicDynamic { a, b } => (bodies[a].entity, bodies[b].entity),
        };
        contacts.push(InitialContact { a, b, manifold });
        if pair.both_solid {
            applied |= apply_correction(bodies, pair, &manifold);
        }
    }

    let mut iterations_run = 1;
    while applied && iterations_run < SOLVER_ITERATIONS {
        applied = false;
        for pair in pairs.iter().filter(|p| p.both_solid) {
            if let Some(manifold) = pair_manifold(bodies, statics, pair) {
                applied |= apply_correction(bodies, pair, &manifold);
            }
        }
        iterations_run += 1;
    }
    SolveOutcome { iterations_run }
}

/// The system: solve, emit pass-0 events, write each moved Transform exactly once.
pub fn resolve_collisions(
    mut bodies: ResMut<DynamicBodies>,
    pairs: Res<CollisionPairs>,
    statics: Res<StaticColliderIndex>,
    mut writer: MessageWriter<CollisionEvent>,
    mut transforms: Query<&mut Transform>,
    mut contacts: Local<Vec<InitialContact>>,
) {
    contacts.clear();
    solve_pairs(&mut bodies.bodies, &statics, &pairs.pairs, &mut contacts);
    for contact in contacts.drain(..) {
        writer.write(CollisionEvent {
            a: contact.a,
            b: contact.b,
            normal: contact.manifold.normal,
            depth: contact.manifold.depth,
        });
    }
    for body in &bodies.bodies {
        if body.offset.length_squared() > COLLISION_EPSILON * COLLISION_EPSILON
            && let Ok(mut transform) = transforms.get_mut(body.entity)
        {
            transform.translation.x += body.offset.x;
            transform.translation.y += body.offset.y;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::Collider;
    use crate::shape::world_aabb;
    use crate::static_index::StaticEntry;
    use crate::world::to_world;

    fn dynamic_box(pos: Vec2, half: Vec2, solid: bool) -> DynamicBody {
        let collider = Collider::obb(half);
        let tx = Transform::from_xyz(pos.x, pos.y, 0.0);
        DynamicBody {
            entity: Entity::PLACEHOLDER,
            shape: to_world(&collider.shape, &tx),
            aabb: world_aabb(&collider.shape, &tx),
            solid,
            offset: Vec2::ZERO,
        }
    }

    fn static_box(pos: Vec2, half: Vec2) -> StaticColliderIndex {
        let mut index = StaticColliderIndex::default();
        index.entries.push(static_entry(pos, half));
        index
    }

    fn static_entry(pos: Vec2, half: Vec2) -> StaticEntry {
        let collider = Collider::obb(half);
        let tx = Transform::from_xyz(pos.x, pos.y, 0.0);
        StaticEntry {
            entity: Entity::PLACEHOLDER,
            shape: to_world(&collider.shape, &tx),
            aabb: world_aabb(&collider.shape, &tx),
            solid: true,
        }
    }

    fn solid_pair(bodies: PairBodies) -> CandidatePair {
        CandidatePair {
            bodies,
            both_solid: true,
        }
    }

    fn residual_depth(
        bodies: &[DynamicBody],
        statics: &StaticColliderIndex,
        pair: &CandidatePair,
    ) -> f32 {
        pair_manifold(bodies, statics, pair).map_or(0.0, |m| m.depth)
    }

    const SETTLE: f32 = PENETRATION_SLOP + 0.05;

    #[test]
    fn dynamic_pushed_out_of_static_opposite_normal() {
        // Dynamic at origin, static box to its right overlapping by 5 — dynamic must move left.
        let mut bodies = vec![dynamic_box(Vec2::ZERO, Vec2::splat(10.0), true)];
        let statics = static_box(Vec2::new(15.0, 0.0), Vec2::splat(10.0));
        let pairs = vec![solid_pair(PairBodies::DynamicStatic { body: 0, anchor: 0 })];
        let mut contacts = Vec::new();
        solve_pairs(&mut bodies, &statics, &pairs, &mut contacts);

        assert!(bodies[0].offset.x < 0.0, "offset {:?}", bodies[0].offset);
        assert!(bodies[0].offset.y.abs() < 1e-4);
        assert!(residual_depth(&bodies, &statics, &pairs[0]) <= SETTLE);
        assert_eq!(contacts.len(), 1);
        assert!(
            contacts[0].manifold.normal.x > 0.9,
            "normal points dyn→static"
        );
        assert!(
            (contacts[0].manifold.depth - 5.0).abs() < 1e-3,
            "pre-resolution depth"
        );
    }

    #[test]
    fn corner_two_statics_resolves_both() {
        // Dynamic box overlapping a "floor" below and a "wall" left by 5 each.
        let mut bodies = vec![dynamic_box(Vec2::ZERO, Vec2::splat(10.0), true)];
        let mut statics = static_box(Vec2::new(-15.0, 0.0), Vec2::splat(10.0));
        statics
            .entries
            .push(static_entry(Vec2::new(0.0, -15.0), Vec2::splat(10.0)));
        let pairs = vec![
            solid_pair(PairBodies::DynamicStatic { body: 0, anchor: 0 }),
            solid_pair(PairBodies::DynamicStatic { body: 0, anchor: 1 }),
        ];
        let mut contacts = Vec::new();
        solve_pairs(&mut bodies, &statics, &pairs, &mut contacts);

        assert!(residual_depth(&bodies, &statics, &pairs[0]) <= SETTLE);
        assert!(residual_depth(&bodies, &statics, &pairs[1]) <= SETTLE);
        assert!(
            bodies[0].offset.x > 0.0 && bodies[0].offset.y > 0.0,
            "pushed diagonally out"
        );
    }

    #[test]
    fn chain_push_propagates_to_static() {
        // A overlaps B by 5, B overlaps static C by 5: corrections must flow through B.
        let mut bodies = vec![
            dynamic_box(Vec2::ZERO, Vec2::splat(10.0), true),
            dynamic_box(Vec2::new(15.0, 0.0), Vec2::splat(10.0), true),
        ];
        let statics = static_box(Vec2::new(30.0, 0.0), Vec2::splat(10.0));
        let pairs = vec![
            solid_pair(PairBodies::DynamicDynamic { a: 0, b: 1 }),
            solid_pair(PairBodies::DynamicStatic { body: 1, anchor: 0 }),
        ];
        let mut contacts = Vec::new();
        let outcome = solve_pairs(&mut bodies, &statics, &pairs, &mut contacts);

        assert!(residual_depth(&bodies, &statics, &pairs[0]) <= SETTLE);
        assert!(residual_depth(&bodies, &statics, &pairs[1]) <= SETTLE);
        assert!(
            bodies[0].offset.x < bodies[1].offset.x,
            "A backs off more than B"
        );
        assert!(bodies[1].offset.x < 0.0, "B pushed away from the static");
        assert!(outcome.iterations_run > 1, "chain needs propagation passes");
    }

    #[test]
    fn separated_pair_pass_zero_only() {
        let mut bodies = vec![dynamic_box(Vec2::ZERO, Vec2::splat(10.0), true)];
        let statics = static_box(Vec2::new(100.0, 0.0), Vec2::splat(10.0));
        let pairs = vec![solid_pair(PairBodies::DynamicStatic { body: 0, anchor: 0 })];
        let mut contacts = Vec::new();
        let outcome = solve_pairs(&mut bodies, &statics, &pairs, &mut contacts);

        assert_eq!(outcome.iterations_run, 1);
        assert!(contacts.is_empty());
        assert_eq!(bodies[0].offset, Vec2::ZERO);
    }

    #[test]
    fn overlap_within_slop_not_corrected() {
        // Penetration 0.3 < slop 0.5: contact reported, nothing moved (rest-jitter kill).
        let mut bodies = vec![dynamic_box(Vec2::ZERO, Vec2::splat(10.0), true)];
        let statics = static_box(Vec2::new(19.7, 0.0), Vec2::splat(10.0));
        let pairs = vec![solid_pair(PairBodies::DynamicStatic { body: 0, anchor: 0 })];
        let mut contacts = Vec::new();
        let outcome = solve_pairs(&mut bodies, &statics, &pairs, &mut contacts);

        assert_eq!(contacts.len(), 1, "still a contact event");
        assert_eq!(bodies[0].offset, Vec2::ZERO);
        assert_eq!(outcome.iterations_run, 1);
    }

    #[test]
    fn symmetric_dynamic_pair_splits_evenly() {
        let mut bodies = vec![
            dynamic_box(Vec2::ZERO, Vec2::splat(10.0), true),
            dynamic_box(Vec2::new(15.0, 0.0), Vec2::splat(10.0), true),
        ];
        let statics = StaticColliderIndex::default();
        let pairs = vec![solid_pair(PairBodies::DynamicDynamic { a: 0, b: 1 })];
        let mut contacts = Vec::new();
        solve_pairs(&mut bodies, &statics, &pairs, &mut contacts);

        assert!(
            (bodies[0].offset.x + bodies[1].offset.x).abs() < 1e-4,
            "equal and opposite"
        );
        assert!(bodies[0].offset.x < 0.0 && bodies[1].offset.x > 0.0);
        assert!(residual_depth(&bodies, &statics, &pairs[0]) <= SETTLE);
    }

    #[test]
    fn non_solid_overlap_reported_not_resolved() {
        let mut bodies = vec![
            dynamic_box(Vec2::ZERO, Vec2::splat(10.0), false), // sensor-style, not solid
            dynamic_box(Vec2::new(15.0, 0.0), Vec2::splat(10.0), true),
        ];
        let statics = StaticColliderIndex::default();
        let pairs = vec![CandidatePair {
            bodies: PairBodies::DynamicDynamic { a: 0, b: 1 },
            both_solid: false,
        }];
        let mut contacts = Vec::new();
        solve_pairs(&mut bodies, &statics, &pairs, &mut contacts);

        assert_eq!(contacts.len(), 1);
        assert!(contacts[0].manifold.normal.x > 0.9, "a→b convention");
        assert!((contacts[0].manifold.depth - 5.0).abs() < 1e-3);
        assert_eq!(bodies[0].offset, Vec2::ZERO);
        assert_eq!(bodies[1].offset, Vec2::ZERO);
    }
}
