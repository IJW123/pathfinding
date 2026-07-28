use bevy::prelude::*;

use logistics::cargo_handling::components::Carrier;
use logistics::components::{Capacity, Inventory};
use range::components::RangeRing;

use crate::components::Station;
use crate::constants::STATION_Z;

/// A station at `pos`, its platform aligned to the track by `facing` (the along-track tangent angle,
/// radians), with catchment `radius`. Holds no goods of its own — it brokers between a mover (the
/// player or a train) in range and the warehouses in range. The catchment is a [`RangeRing`]: the
/// radius is world-logic data the haul system reads *and* the renderer draws as a disc. The transform
/// (placed at [`STATION_Z`]) also orients the rendered platform bar.
#[must_use]
pub fn station(pos: Vec2, facing: f32, radius: f32) -> impl Bundle {
    let transform = Transform::from_translation(pos.extend(STATION_Z))
        .with_rotation(Quat::from_rotation_z(facing));
    (Station, transform, RangeRing { radius })
}

/// The components that turn a locomotive into a cargo train: a [`Carrier`] marker plus an empty
/// [`Inventory`] capped by `capacity`. Inserted onto the loco entity — kept here, not in `rail`'s
/// bundle, so `rail` stays free of `logistics`. The `Carrier` marker is how the station broker finds
/// the train (the same marker the player carries), so one path serves both.
#[must_use]
pub fn cargo_train(capacity: Capacity) -> impl Bundle {
    (Carrier, Inventory::default(), capacity)
}
