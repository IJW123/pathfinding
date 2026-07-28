use bevy::prelude::*;

/// A cargo transfer node. Holds no goods; brokers cargo between a
/// [`Carrier`](logistics::cargo_handling::components::Carrier) mover inside its
/// [`RangeRing`](range::components::RangeRing) catchment and the
/// [`Storage`](logistics::components::Storage) buildings in that same radius.
#[derive(Component)]
pub struct Station;
