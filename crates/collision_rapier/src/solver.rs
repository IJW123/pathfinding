use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use rapier2d::geometry::Collider as RapierCollider;
use rapier2d::math::Pose;
use rapier2d::parry::query;
use rapier2d::parry::shape::SharedShape;

use hitboxes_rapier::components::Solid;
use hitboxes_rapier::convert::{parry_to_vec2, vec2_to_parry};

use crate::constants::{
    COLLISION_EPSILON, PENETRATION_PERCENT, PENETRATION_SLOP, SOLVER_ITERATIONS,
};
use crate::events::CollisionEvent;
use crate::mapping::{ColliderMap, entity_from_user_data};
use crate::physics::PhysicsWorld;

/// Per-tick snapshot of one dynamic collider. `pose` is the world pose rapier stepped with;
/// `offset` accumulates corrections for the single Transform write-back at the end of the tick
/// (applied on top of `pose` at every contact re-test, so cached geometry stays exact).
pub struct SolverBody {
    pub entity: Entity,
    pub pose: Pose,
    pub shape: SharedShape,
    pub offset: Vec2,
}

/// World pose + shape of one static collider touching a dynamic this tick.
pub struct StaticAnchor {
    pub entity: Entity,
    pub pose: Pose,
    pub shape: SharedShape,
}

/// Who a candidate pair connects. Indices into the gathered body/anchor lists. The dynamic
/// side (or the first-gathered dynamic) is always `a`, so normals point a→b.
pub enum PairBodies {
    DynamicDynamic { a: usize, b: usize },
    DynamicStatic { body: usize, anchor: usize },
}

pub struct CandidatePair {
    pub bodies: PairBodies,
    pub both_solid: bool,
}

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
    pub normal: Vec2,
    pub depth: f32,
}

pub struct SolveOutcome {
    pub iterations_run: usize,
}

fn shifted(pose: &Pose, offset: Vec2) -> Pose {
    Pose {
        rotation: pose.rotation,
        translation: pose.translation + vec2_to_parry(offset),
    }
}

/// Penetrating contact for the current (offset-corrected) poses. parry's `normal1` points
/// shape1→shape2, which is exactly the old manifold's a→b convention (pinned by test).
/// Touching within `COLLISION_EPSILON` counts as separated, like the old SAT gate.
fn touching_contact(
    pose_a: &Pose,
    shape_a: &SharedShape,
    pose_b: &Pose,
    shape_b: &SharedShape,
) -> Option<(Vec2, f32)> {
    let contact = query::contact(pose_a, &**shape_a, pose_b, &**shape_b, 0.0)
        .ok()
        .flatten()?;
    (contact.dist < -COLLISION_EPSILON).then(|| (parry_to_vec2(contact.normal1), -contact.dist))
}

/// Contact for a candidate pair from current poses. Normal points a→b (dynamic→static for
/// mixed pairs). Re-queried every pass so corrections propagate through chains.
fn pair_contact(
    bodies: &[SolverBody],
    anchors: &[StaticAnchor],
    pair: &CandidatePair,
) -> Option<(Vec2, f32)> {
    match pair.bodies {
        PairBodies::DynamicStatic { body, anchor } => {
            let body = &bodies[body];
            let anchor = &anchors[anchor];
            touching_contact(
                &shifted(&body.pose, body.offset),
                &body.shape,
                &anchor.pose,
                &anchor.shape,
            )
        }
        PairBodies::DynamicDynamic { a, b } => {
            let (a, b) = (&bodies[a], &bodies[b]);
            touching_contact(
                &shifted(&a.pose, a.offset),
                &a.shape,
                &shifted(&b.pose, b.offset),
                &b.shape,
            )
        }
    }
}

/// Correction for one contact: resolve penetration beyond `slop`, damped by
/// `PENETRATION_PERCENT`. Returns false when the contact is already within slop. `slop` is passed
/// in (not read from the constant) so the dead-band is testable independent of the production value.
fn apply_correction(
    bodies: &mut [SolverBody],
    pair: &CandidatePair,
    normal: Vec2,
    depth: f32,
    slop: f32,
) -> bool {
    let push_len = (depth - slop).max(0.0) * PENETRATION_PERCENT;
    if push_len <= 0.0 {
        return false;
    }
    let push = normal * push_len;
    match pair.bodies {
        PairBodies::DynamicStatic { body, .. } => {
            // Static side never moves; the dynamic `a` takes the full correction against the normal.
            let weights = pair_weights(false, true);
            bodies[body].offset -= push * weights.a;
        }
        PairBodies::DynamicDynamic { a, b } => {
            let weights = pair_weights(false, false);
            bodies[a].offset -= push * weights.a;
            bodies[b].offset += push * weights.b;
        }
    }
    true
}

/// Gauss-Seidel positional solver. Pass 0 tests every candidate pair, records pre-resolution
/// contacts (event emission preserves today's semantics: any touching collider pair), and
/// corrects solid pairs immediately. Passes 1.. re-test only solid pairs against *current*
/// positions — corrections propagate through chains one link per pass — and stop early once a
/// pass applies nothing. Corrections accumulate in `SolverBody::offset`; Transforms are
/// untouched here.
pub fn solve_pairs(
    bodies: &mut [SolverBody],
    anchors: &[StaticAnchor],
    pairs: &[CandidatePair],
    contacts: &mut Vec<InitialContact>,
    slop: f32,
) -> SolveOutcome {
    let mut applied = false;
    for pair in pairs {
        let Some((normal, depth)) = pair_contact(bodies, anchors, pair) else {
            continue;
        };
        let (a, b) = match pair.bodies {
            PairBodies::DynamicStatic { body, anchor } => {
                (bodies[body].entity, anchors[anchor].entity)
            }
            PairBodies::DynamicDynamic { a, b } => (bodies[a].entity, bodies[b].entity),
        };
        contacts.push(InitialContact {
            a,
            b,
            normal,
            depth,
        });
        if pair.both_solid {
            applied |= apply_correction(bodies, pair, normal, depth, slop);
        }
    }

    let mut iterations_run = 1;
    while applied && iterations_run < SOLVER_ITERATIONS {
        applied = false;
        for pair in pairs.iter().filter(|p| p.both_solid) {
            if let Some((normal, depth)) = pair_contact(bodies, anchors, pair) {
                applied |= apply_correction(bodies, pair, normal, depth, slop);
            }
        }
        iterations_run += 1;
    }
    SolveOutcome { iterations_run }
}

/// Solver inputs gathered from rapier's contact graph. Rebuilt every tick; allocations retained.
#[derive(Default)]
pub struct SolveBuffers {
    bodies: Vec<SolverBody>,
    anchors: Vec<StaticAnchor>,
    pairs: Vec<CandidatePair>,
    body_index: HashMap<Entity, usize>,
    anchor_index: HashMap<Entity, usize>,
    contacts: Vec<InitialContact>,
}

fn body_slot(
    bodies: &mut Vec<SolverBody>,
    index: &mut HashMap<Entity, usize>,
    entity: Entity,
    collider: &RapierCollider,
) -> usize {
    *index.entry(entity).or_insert_with(|| {
        bodies.push(SolverBody {
            entity,
            pose: *collider.position(),
            shape: collider.shared_shape().clone(),
            offset: Vec2::ZERO,
        });
        bodies.len() - 1
    })
}

fn anchor_slot(
    anchors: &mut Vec<StaticAnchor>,
    index: &mut HashMap<Entity, usize>,
    entity: Entity,
    collider: &RapierCollider,
) -> usize {
    *index.entry(entity).or_insert_with(|| {
        anchors.push(StaticAnchor {
            entity,
            pose: *collider.position(),
            shape: collider.shared_shape().clone(),
        });
        anchors.len() - 1
    })
}

/// Candidate pairs from rapier's contact graph (every edge = bounding volumes within the
/// prediction distance, the old margin-inflated list). The dynamic side is gathered as `a`;
/// both-static edges are skipped outright.
fn gather_candidates(
    physics: &PhysicsWorld,
    map: &ColliderMap,
    solids: &Query<Has<Solid>>,
    buffers: &mut SolveBuffers,
) {
    for pair in physics.narrow_phase.contact_graph().interactions() {
        let (Some(first), Some(second)) = (
            physics.colliders.get(pair.collider1),
            physics.colliders.get(pair.collider2),
        ) else {
            continue;
        };
        let entity_first = entity_from_user_data(first.user_data);
        let entity_second = entity_from_user_data(second.user_data);
        let first_static = map
            .bindings
            .get(&entity_first)
            .is_none_or(super::mapping::BodyBinding::is_static);
        let second_static = map
            .bindings
            .get(&entity_second)
            .is_none_or(super::mapping::BodyBinding::is_static);
        if first_static && second_static {
            continue;
        }
        let ((entity_a, collider_a), (entity_b, collider_b, b_static)) = if first_static {
            ((entity_second, second), (entity_first, first, first_static))
        } else {
            (
                (entity_first, first),
                (entity_second, second, second_static),
            )
        };
        let both_solid =
            solids.get(entity_a).unwrap_or(false) && solids.get(entity_b).unwrap_or(false);
        let a = body_slot(
            &mut buffers.bodies,
            &mut buffers.body_index,
            entity_a,
            collider_a,
        );
        let bodies = if b_static {
            PairBodies::DynamicStatic {
                body: a,
                anchor: anchor_slot(
                    &mut buffers.anchors,
                    &mut buffers.anchor_index,
                    entity_b,
                    collider_b,
                ),
            }
        } else {
            PairBodies::DynamicDynamic {
                a,
                b: body_slot(
                    &mut buffers.bodies,
                    &mut buffers.body_index,
                    entity_b,
                    collider_b,
                ),
            }
        };
        buffers.pairs.push(CandidatePair { bodies, both_solid });
    }
}

/// The system: gather pairs from the narrow phase, solve, emit pass-0 events, write each moved
/// Transform exactly once.
pub fn resolve_collisions(
    physics: Res<PhysicsWorld>,
    map: Res<ColliderMap>,
    solids: Query<Has<Solid>>,
    mut transforms: Query<&mut Transform>,
    mut writer: MessageWriter<CollisionEvent>,
    mut buffers: Local<SolveBuffers>,
) {
    buffers.bodies.clear();
    buffers.anchors.clear();
    buffers.pairs.clear();
    buffers.body_index.clear();
    buffers.anchor_index.clear();
    buffers.contacts.clear();

    gather_candidates(&physics, &map, &solids, &mut buffers);
    let SolveBuffers {
        bodies,
        anchors,
        pairs,
        contacts,
        ..
    } = &mut *buffers;
    solve_pairs(bodies, anchors, pairs, contacts, PENETRATION_SLOP);

    for contact in buffers.contacts.drain(..) {
        writer.write(CollisionEvent {
            a: contact.a,
            b: contact.b,
            normal: contact.normal,
            depth: contact.depth,
        });
    }
    for body in &buffers.bodies {
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
    use hitboxes_rapier::components::Collider;
    use hitboxes_rapier::convert::transform_to_pose;
    use rapier2d::parry::shape::{Ball, ConvexPolygon, Cuboid};

    /// Pins the assumption the whole solver rests on: parry's `Contact::normal1` points from
    /// the first shape toward the second when penetrating, for every shape pairing we use.
    #[test]
    fn parry_contact_normal_points_a_to_b() {
        let origin = Pose::identity();
        let right = Pose::new(vec2_to_parry(Vec2::new(15.0, 0.0)), 0.0);

        let ball_a = SharedShape::new(Ball::new(10.0));
        let ball_b = SharedShape::new(Ball::new(10.0));
        let (normal, depth) =
            touching_contact(&origin, &ball_a, &right, &ball_b).expect("penetrating");
        assert!(normal.x > 0.9, "ball-ball normal a→b, got {normal:?}");
        assert!((depth - 5.0).abs() < 1e-3);

        let box_a = SharedShape::new(Cuboid::new(vec2_to_parry(Vec2::splat(10.0))));
        let box_b = SharedShape::new(Cuboid::new(vec2_to_parry(Vec2::splat(10.0))));
        let (normal, depth) =
            touching_contact(&origin, &box_a, &right, &box_b).expect("penetrating");
        assert!(normal.x > 0.9, "box-box normal a→b, got {normal:?}");
        assert!((depth - 5.0).abs() < 1e-3);

        let poly = SharedShape::new(
            ConvexPolygon::from_convex_hull(&[
                vec2_to_parry(Vec2::new(-10.0, -10.0)),
                vec2_to_parry(Vec2::new(10.0, -10.0)),
                vec2_to_parry(Vec2::new(10.0, 10.0)),
                vec2_to_parry(Vec2::new(-10.0, 10.0)),
            ])
            .expect("valid hull"),
        );
        let (normal, _) = touching_contact(&origin, &poly, &right, &ball_b).expect("penetrating");
        assert!(normal.x > 0.9, "poly-ball normal a→b, got {normal:?}");
        // Swapped order negates the normal.
        let (normal, _) = touching_contact(&right, &ball_b, &origin, &poly).expect("penetrating");
        assert!(normal.x < -0.9, "swap negates normal, got {normal:?}");
    }

    fn obb_body(pos: Vec2, half: Vec2) -> SolverBody {
        let collider = Collider::obb(half);
        SolverBody {
            entity: Entity::PLACEHOLDER,
            pose: transform_to_pose(&Transform::from_xyz(pos.x, pos.y, 0.0)),
            shape: collider.shape.to_shared_shape(),
            offset: Vec2::ZERO,
        }
    }

    fn obb_anchor(pos: Vec2, half: Vec2) -> StaticAnchor {
        let collider = Collider::obb(half);
        StaticAnchor {
            entity: Entity::PLACEHOLDER,
            pose: transform_to_pose(&Transform::from_xyz(pos.x, pos.y, 0.0)),
            shape: collider.shape.to_shared_shape(),
        }
    }

    fn solid_pair(bodies: PairBodies) -> CandidatePair {
        CandidatePair {
            bodies,
            both_solid: true,
        }
    }

    fn residual_depth(
        bodies: &[SolverBody],
        anchors: &[StaticAnchor],
        pair: &CandidatePair,
    ) -> f32 {
        pair_contact(bodies, anchors, pair).map_or(0.0, |(_, depth)| depth)
    }

    const SETTLE: f32 = PENETRATION_SLOP + 0.05;

    #[test]
    fn dynamic_pushed_out_of_static_opposite_normal() {
        // Dynamic at origin, static box to its right overlapping by 5 — dynamic must move left.
        let mut bodies = vec![obb_body(Vec2::ZERO, Vec2::splat(10.0))];
        let anchors = vec![obb_anchor(Vec2::new(15.0, 0.0), Vec2::splat(10.0))];
        let pairs = vec![solid_pair(PairBodies::DynamicStatic { body: 0, anchor: 0 })];
        let mut contacts = Vec::new();
        solve_pairs(
            &mut bodies,
            &anchors,
            &pairs,
            &mut contacts,
            PENETRATION_SLOP,
        );

        assert!(bodies[0].offset.x < 0.0, "offset {:?}", bodies[0].offset);
        assert!(bodies[0].offset.y.abs() < 1e-4);
        assert!(residual_depth(&bodies, &anchors, &pairs[0]) <= SETTLE);
        assert_eq!(contacts.len(), 1);
        assert!(contacts[0].normal.x > 0.9, "normal points dyn→static");
        assert!(
            (contacts[0].depth - 5.0).abs() < 1e-3,
            "pre-resolution depth"
        );
    }

    #[test]
    fn corner_two_statics_resolves_both() {
        // Dynamic box overlapping a "floor" below and a "wall" left by 5 each.
        let mut bodies = vec![obb_body(Vec2::ZERO, Vec2::splat(10.0))];
        let anchors = vec![
            obb_anchor(Vec2::new(-15.0, 0.0), Vec2::splat(10.0)),
            obb_anchor(Vec2::new(0.0, -15.0), Vec2::splat(10.0)),
        ];
        let pairs = vec![
            solid_pair(PairBodies::DynamicStatic { body: 0, anchor: 0 }),
            solid_pair(PairBodies::DynamicStatic { body: 0, anchor: 1 }),
        ];
        let mut contacts = Vec::new();
        solve_pairs(
            &mut bodies,
            &anchors,
            &pairs,
            &mut contacts,
            PENETRATION_SLOP,
        );

        assert!(residual_depth(&bodies, &anchors, &pairs[0]) <= SETTLE);
        assert!(residual_depth(&bodies, &anchors, &pairs[1]) <= SETTLE);
        assert!(
            bodies[0].offset.x > 0.0 && bodies[0].offset.y > 0.0,
            "pushed diagonally out"
        );
    }

    #[test]
    fn chain_push_propagates_to_static() {
        // A overlaps B by 5, B overlaps static C by 5: corrections must flow through B.
        let mut bodies = vec![
            obb_body(Vec2::ZERO, Vec2::splat(10.0)),
            obb_body(Vec2::new(15.0, 0.0), Vec2::splat(10.0)),
        ];
        let anchors = vec![obb_anchor(Vec2::new(30.0, 0.0), Vec2::splat(10.0))];
        let pairs = vec![
            solid_pair(PairBodies::DynamicDynamic { a: 0, b: 1 }),
            solid_pair(PairBodies::DynamicStatic { body: 1, anchor: 0 }),
        ];
        let mut contacts = Vec::new();
        let outcome = solve_pairs(
            &mut bodies,
            &anchors,
            &pairs,
            &mut contacts,
            PENETRATION_SLOP,
        );

        assert!(residual_depth(&bodies, &anchors, &pairs[0]) <= SETTLE);
        assert!(residual_depth(&bodies, &anchors, &pairs[1]) <= SETTLE);
        assert!(
            bodies[0].offset.x < bodies[1].offset.x,
            "A backs off more than B"
        );
        assert!(bodies[1].offset.x < 0.0, "B pushed away from the static");
        assert!(outcome.iterations_run > 1, "chain needs propagation passes");
    }

    #[test]
    fn separated_pair_pass_zero_only() {
        let mut bodies = vec![obb_body(Vec2::ZERO, Vec2::splat(10.0))];
        let anchors = vec![obb_anchor(Vec2::new(100.0, 0.0), Vec2::splat(10.0))];
        let pairs = vec![solid_pair(PairBodies::DynamicStatic { body: 0, anchor: 0 })];
        let mut contacts = Vec::new();
        let outcome = solve_pairs(
            &mut bodies,
            &anchors,
            &pairs,
            &mut contacts,
            PENETRATION_SLOP,
        );

        assert_eq!(outcome.iterations_run, 1);
        assert!(contacts.is_empty());
        assert_eq!(bodies[0].offset, Vec2::ZERO);
    }

    #[test]
    fn slop_band_gates_corrections() {
        // Tests the slop dead-band directly with an explicit slop, independent of the production
        // PENETRATION_SLOP value: penetration below slop is left in place (rest-jitter kill);
        // penetration beyond slop is projected down to the slop band.
        const TEST_SLOP: f32 = 0.5;

        // Shallow: penetration 0.3 < slop → contact reported, nothing moved, no extra passes.
        let mut bodies = vec![obb_body(Vec2::ZERO, Vec2::splat(10.0))];
        let anchors = vec![obb_anchor(Vec2::new(19.7, 0.0), Vec2::splat(10.0))];
        let pairs = vec![solid_pair(PairBodies::DynamicStatic { body: 0, anchor: 0 })];
        let mut contacts = Vec::new();
        let outcome = solve_pairs(&mut bodies, &anchors, &pairs, &mut contacts, TEST_SLOP);

        assert_eq!(contacts.len(), 1, "still a contact event");
        assert_eq!(
            bodies[0].offset,
            Vec2::ZERO,
            "shallow overlap left in place"
        );
        assert_eq!(outcome.iterations_run, 1, "no correction passes");

        // Deep: penetration 1.0 > slop → corrected, settling to ~slop (not fully out).
        let mut bodies = vec![obb_body(Vec2::ZERO, Vec2::splat(10.0))];
        let anchors = vec![obb_anchor(Vec2::new(19.0, 0.0), Vec2::splat(10.0))];
        let pairs = vec![solid_pair(PairBodies::DynamicStatic { body: 0, anchor: 0 })];
        let mut contacts = Vec::new();
        solve_pairs(&mut bodies, &anchors, &pairs, &mut contacts, TEST_SLOP);

        assert!(bodies[0].offset.x < 0.0, "deep overlap corrected");
        let residual = residual_depth(&bodies, &anchors, &pairs[0]);
        assert!(
            (residual - TEST_SLOP).abs() < 1e-3,
            "settles to the slop band, not fully out: got {residual}"
        );
    }

    #[test]
    fn symmetric_dynamic_pair_splits_evenly() {
        let mut bodies = vec![
            obb_body(Vec2::ZERO, Vec2::splat(10.0)),
            obb_body(Vec2::new(15.0, 0.0), Vec2::splat(10.0)),
        ];
        let anchors = Vec::new();
        let pairs = vec![solid_pair(PairBodies::DynamicDynamic { a: 0, b: 1 })];
        let mut contacts = Vec::new();
        solve_pairs(
            &mut bodies,
            &anchors,
            &pairs,
            &mut contacts,
            PENETRATION_SLOP,
        );

        assert!(
            (bodies[0].offset.x + bodies[1].offset.x).abs() < 1e-4,
            "equal and opposite"
        );
        assert!(bodies[0].offset.x < 0.0 && bodies[1].offset.x > 0.0);
        assert!(residual_depth(&bodies, &anchors, &pairs[0]) <= SETTLE);
    }

    #[test]
    fn non_solid_overlap_reported_not_resolved() {
        let mut bodies = vec![
            obb_body(Vec2::ZERO, Vec2::splat(10.0)),
            obb_body(Vec2::new(15.0, 0.0), Vec2::splat(10.0)),
        ];
        let anchors = Vec::new();
        let pairs = vec![CandidatePair {
            bodies: PairBodies::DynamicDynamic { a: 0, b: 1 },
            both_solid: false,
        }];
        let mut contacts = Vec::new();
        solve_pairs(
            &mut bodies,
            &anchors,
            &pairs,
            &mut contacts,
            PENETRATION_SLOP,
        );

        assert_eq!(contacts.len(), 1);
        assert!(contacts[0].normal.x > 0.9, "a→b convention");
        assert!((contacts[0].depth - 5.0).abs() < 1e-3);
        assert_eq!(bodies[0].offset, Vec2::ZERO);
        assert_eq!(bodies[1].offset, Vec2::ZERO);
    }
}
