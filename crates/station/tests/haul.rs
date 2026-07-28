//! `station_haul_input` emits the right `CargoHaul` when a selected mover is parked in a station's
//! radius, and stays silent when it isn't.

use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;

use logistics::cargo_handling::components::Carrier;
use logistics::cargo_handling::message::CargoHaul;
use logistics::components::{Capacity, Inventory, Storage};
use selection::components::Selected;
use station::bundle::station;
use station::constants::LOAD_KEY;
use station::systems::station_haul_input;

/// A `World` with the `CargoHaul` message resource (the system's `MessageWriter` needs it) and a
/// `ButtonInput<KeyCode>` with `key` pressed (sets `just_pressed`).
fn world_with_key(key: KeyCode) -> World {
    let mut world = World::new();
    world.init_resource::<Messages<CargoHaul>>();
    let mut input = ButtonInput::<KeyCode>::default();
    input.press(key);
    world.insert_resource(input);
    world
}

/// A selected cargo mover (a `Carrier`) at `pos` — stands in for the player or a train.
fn spawn_train(world: &mut World, pos: Vec2) -> Entity {
    world
        .spawn((
            Carrier,
            Selected,
            Transform::from_translation(pos.extend(0.0)),
            Inventory::default(),
            Capacity::default(),
        ))
        .id()
}

fn spawn_station(world: &mut World, pos: Vec2, radius: f32) {
    world.spawn(station(pos, 0.0, radius));
}

fn spawn_storage(world: &mut World, pos: Vec2) -> Entity {
    world
        .spawn((Storage, Transform::from_translation(pos.extend(0.0))))
        .id()
}

fn drain_hauls(world: &mut World) -> Vec<(Entity, Entity)> {
    world
        .resource_mut::<Messages<CargoHaul>>()
        .drain()
        .map(|h| (h.source, h.dest))
        .collect()
}

#[test]
fn load_emits_haul_from_in_range_storage_to_train() {
    let mut world = world_with_key(LOAD_KEY);
    let train = spawn_train(&mut world, Vec2::ZERO);
    spawn_station(&mut world, Vec2::ZERO, 200.0);
    let storage = spawn_storage(&mut world, Vec2::new(50.0, 0.0)); // inside the radius

    world
        .run_system_once(station_haul_input)
        .expect("system runs");

    // Load pulls warehouse → train.
    assert_eq!(drain_hauls(&mut world), vec![(storage, train)]);
}

#[test]
fn train_outside_every_station_emits_nothing() {
    let mut world = world_with_key(LOAD_KEY);
    spawn_train(&mut world, Vec2::new(10_000.0, 0.0)); // far from any station
    spawn_station(&mut world, Vec2::ZERO, 200.0);
    spawn_storage(&mut world, Vec2::new(50.0, 0.0));

    world
        .run_system_once(station_haul_input)
        .expect("system runs");

    assert!(drain_hauls(&mut world).is_empty());
}

#[test]
fn storage_outside_the_station_radius_is_not_brokered() {
    // Train is at the station, but the only warehouse is beyond the catchment ⇒ nothing to broker.
    let mut world = world_with_key(LOAD_KEY);
    spawn_train(&mut world, Vec2::ZERO);
    spawn_station(&mut world, Vec2::ZERO, 200.0);
    spawn_storage(&mut world, Vec2::new(1_000.0, 0.0)); // outside the 200 radius

    world
        .run_system_once(station_haul_input)
        .expect("system runs");

    assert!(drain_hauls(&mut world).is_empty());
}
