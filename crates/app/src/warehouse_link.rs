//! Demo wiring: a second warehouse linked to the authored one by a grade-routed rail. This is the
//! first end-to-end exercise of `routing::find_path` against the live `HeightField` — `app` is the
//! only crate with `world` + `routing` + `rail` together, which the routing plan named as the home
//! for this seam. Promote into authored level data once the feature graduates past a demo.

use bevy::prelude::*;

use level::objects::spec::{LevelSpec, StorageSpec};
use level::objects::storage::storage;
use logistics::commodity::Commodity;
use rail::bundle::{locomotive, rail_track};
use rail::components::RailHeading;
use rail::track::RailTrack;
use routing::{PathProfile, find_path};
use world::elevation::height_field::HeightField;

/// Where the linked warehouse sits — across the rolling-hill terrain from the authored warehouse, so
/// the route has to bend around features rather than run straight.
const WAREHOUSE_B_POS: Vec2 = Vec2::new(1100.0, -500.0);
/// Stock seeded into the linked warehouse (just so it isn't empty).
const WAREHOUSE_B_STOCK: u32 = 30;
/// Match the authored warehouse's footprint/dock so the pair reads as the same kind of building.
const WAREHOUSE_HALF_EXTENT: f32 = 50.0;
const WAREHOUSE_MAX_VOLUME: f32 = 20.0;
const WAREHOUSE_DOCK_RADIUS: f32 = 120.0;

/// Rail routing profile: steepest tolerated grade per step (`|Δh|/horizontal`), how hard to prefer
/// flatter ground, and the search grid spacing. Generous grade for a demo over hills; real rail
/// would be stricter.
const RAIL_MAX_GRADE: f32 = 0.1;
const RAIL_GRADE_COST_WEIGHT: f32 = 1.0;
const RAIL_STEP: f32 = 10.0;

fn rail_profile() -> PathProfile {
    PathProfile {
        max_grade: RAIL_MAX_GRADE,
        grade_cost_weight: RAIL_GRADE_COST_WEIGHT,
        step: RAIL_STEP,
    }
}

/// Spawns the linked warehouse and, if the terrain allows a grade-legal route, the rail connecting it
/// to the authored warehouse plus a locomotive sitting at the start of that rail.
pub struct WarehouseLinkPlugin;

impl Plugin for WarehouseLinkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, link_warehouses);
    }
}

/// Reads the authored warehouse position from [`LevelSpec`] and the live [`HeightField`], drops a
/// second warehouse at [`WAREHOUSE_B_POS`], and routes a rail between the two. Runs at `Startup`: the
/// height field is built in `PreStartup`, the spec is inserted at plugin build, so both are ready.
fn link_warehouses(mut commands: Commands, level: Res<LevelSpec>, height: Res<HeightField>) {
    let from = level.storage.pos;
    let to = WAREHOUSE_B_POS;

    commands.spawn(storage(&StorageSpec {
        pos: to,
        half_extent: WAREHOUSE_HALF_EXTENT,
        max_volume: WAREHOUSE_MAX_VOLUME,
        dock_radius: WAREHOUSE_DOCK_RADIUS,
        stock: vec![(Commodity::Coal, WAREHOUSE_B_STOCK)],
    }));

    match find_path(&*height, from, to, &rail_profile()) {
        Some(points) => {
            info!(
                waypoints = points.len(),
                ?from,
                ?to,
                "routed rail between warehouses"
            );
            let track = RailTrack::new(points);
            commands.spawn(locomotive(&track, 0.0, RailHeading::Forward));
            commands.spawn(rail_track(track));
        }
        None => warn!(
            ?from,
            ?to,
            "no grade-legal rail route between the warehouses"
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use level::objects::manifest::RawLevelSpec;

    /// Routes between the two warehouses over the *real* authored terrain, so a regression in either
    /// the recipe or the router (that breaks this demo) fails here rather than silently showing no
    /// rail in the running app.
    #[test]
    fn warehouses_have_a_grade_legal_route() {
        // Cargo runs tests with cwd = crate dir, so the loader's cwd-relative path won't resolve;
        // reach the workspace asset via CARGO_MANIFEST_DIR instead.
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/level.ron");
        let text = std::fs::read_to_string(path).expect("read level.ron");
        let raw: RawLevelSpec = ron::from_str(&text).expect("parse level.ron");
        let spec = LevelSpec::from(raw);
        let field = HeightField::new(&spec.terrain_config());

        let route = find_path(&field, spec.storage.pos, WAREHOUSE_B_POS, &rail_profile())
            .expect("a grade-legal rail route should connect the two warehouses");

        assert!(route.len() >= 2, "route must have at least two points");
        assert_eq!(
            route.first().copied(),
            Some(spec.storage.pos),
            "route starts at warehouse A"
        );
        assert_eq!(
            route.last().copied(),
            Some(WAREHOUSE_B_POS),
            "route ends at warehouse B"
        );
    }
}
