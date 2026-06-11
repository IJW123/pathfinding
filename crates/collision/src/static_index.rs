use bevy::platform::collections::HashMap;
use bevy::prelude::*;

use crate::aabb::Aabb;
use crate::components::{Collider, Solid, Static};
use crate::grid::cells_for_aabb;
use crate::shape::world_aabb;
use crate::world::{WorldShape, to_world};

/// One cached static collider: world-space geometry computed once, valid until the entity's
/// `Transform`/`Collider` changes (statics don't move, so effectively forever).
pub struct StaticEntry {
    pub entity: Entity,
    pub shape: WorldShape,
    pub aabb: Aabb,
    pub solid: bool,
}

/// Spatial index over static colliders. `cells` maps a grid cell to indices into `entries`.
/// Rebuilt only when the static set changes; the broad phase reads it every tick without
/// touching the ECS.
#[derive(Resource, Default)]
pub struct StaticColliderIndex {
    pub cells: HashMap<IVec2, Vec<usize>>,
    pub entries: Vec<StaticEntry>,
}

/// Statics whose collision-relevant data changed this tick (`Changed` covers `Added`).
type ChangedStatics = (
    With<Static>,
    With<Collider>,
    Or<(Changed<Transform>, Changed<Collider>)>,
);

/// Rebuild the index when any static was added/changed, or when the static count shrank
/// (despawn / `Static` removal). Deliberately not `RemovedComponents`: removal events are
/// frame-buffered, and `FixedUpdate` can skip frames at high FPS and miss them entirely —
/// the count comparison is skip-proof. Full rebuild on dirty is fine; statics change rarely.
pub fn maintain_static_index(
    mut index: ResMut<StaticColliderIndex>,
    changed: Query<(), ChangedStatics>,
    all: Query<(Entity, &Transform, &Collider, Has<Solid>), With<Static>>,
) {
    let dirty = !changed.is_empty() || all.iter().count() != index.entries.len();
    if !dirty {
        return;
    }
    index.cells.clear();
    index.entries.clear();
    for (entity, transform, collider, solid) in &all {
        let shape = to_world(&collider.shape, transform);
        let aabb = world_aabb(&collider.shape, transform);
        let entry_idx = index.entries.len();
        index.entries.push(StaticEntry {
            entity,
            shape,
            aabb,
            solid,
        });
        for cell in cells_for_aabb(&aabb) {
            index.cells.entry(cell).or_default().push(entry_idx);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app_with_index() -> App {
        let mut app = App::new();
        app.init_resource::<StaticColliderIndex>()
            .add_systems(Update, maintain_static_index);
        app
    }

    fn spawn_static_box(app: &mut App, pos: Vec2) -> Entity {
        app.world_mut()
            .spawn((
                Transform::from_xyz(pos.x, pos.y, 0.0),
                Collider::obb(Vec2::splat(10.0)),
                Solid,
                Static,
            ))
            .id()
    }

    #[test]
    fn populated_on_first_run() {
        let mut app = app_with_index();
        spawn_static_box(&mut app, Vec2::ZERO);
        app.update();
        let index = app.world().resource::<StaticColliderIndex>();
        assert_eq!(index.entries.len(), 1);
        assert!(index.entries[0].solid);
        assert!(!index.cells.is_empty());
    }

    #[test]
    fn clean_ticks_do_not_rebuild() {
        let mut app = app_with_index();
        spawn_static_box(&mut app, Vec2::ZERO);
        app.update();
        let cached = app.world().resource::<StaticColliderIndex>().entries[0].aabb;
        // Mutate the resource out-of-band; a rebuild would overwrite this sentinel.
        app.world_mut()
            .resource_mut::<StaticColliderIndex>()
            .entries[0]
            .aabb = Aabb {
            min: Vec2::splat(999.0),
            max: Vec2::splat(999.0),
        };
        app.update();
        let index = app.world().resource::<StaticColliderIndex>();
        assert_eq!(index.entries[0].aabb.min, Vec2::splat(999.0));
        assert_ne!(cached.min, Vec2::splat(999.0));
    }

    #[test]
    fn transform_change_rebuilds() {
        let mut app = app_with_index();
        let e = spawn_static_box(&mut app, Vec2::ZERO);
        app.update();
        app.world_mut()
            .entity_mut(e)
            .get_mut::<Transform>()
            .unwrap()
            .translation
            .x = 100.0;
        app.update();
        let index = app.world().resource::<StaticColliderIndex>();
        assert_eq!(index.entries.len(), 1);
        assert!((index.entries[0].aabb.min.x - 90.0).abs() < 1e-4);
    }

    #[test]
    fn despawn_removes_entry_via_count_mismatch() {
        let mut app = app_with_index();
        let e = spawn_static_box(&mut app, Vec2::ZERO);
        spawn_static_box(&mut app, Vec2::new(200.0, 0.0));
        app.update();
        assert_eq!(
            app.world().resource::<StaticColliderIndex>().entries.len(),
            2
        );
        app.world_mut().entity_mut(e).despawn();
        app.update();
        let index = app.world().resource::<StaticColliderIndex>();
        assert_eq!(index.entries.len(), 1);
        assert_ne!(index.entries[0].entity, e);
    }
}
