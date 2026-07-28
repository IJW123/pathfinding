use bevy::prelude::*;

use logistics::cargo_handling::components::Carrier;
use logistics::cargo_handling::message::CargoHaul;
use logistics::components::Storage;
use range::components::RangeRing;
use selection::components::Selected;

use crate::components::Station;
use crate::constants::{LOAD_KEY, UNLOAD_KEY};

/// The cargo mover the player has selected: a [`Carrier`] (the player avatar or a train) that is
/// `Selected`. Aliased to keep the query readable and past clippy's `type_complexity` gate.
type SelectedCarrier = (With<Carrier>, With<Selected>);

/// On a load/unload key press, broker cargo between the selected mover and the warehouses in the
/// station it's parked in. Finds the nearest [`Station`] whose radius contains the mover, then writes
/// a [`CargoHaul`] for every [`Storage`] in that station's radius — load pulls warehouse → mover,
/// unload pushes mover → warehouse. [`apply_cargo_hauls`](logistics) does the actual moving, capping
/// by `Capacity`. No "stopped" check: the player stops, then presses. The mover is any [`Carrier`],
/// so the player avatar and rail trains use one and the same path.
pub fn station_haul_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    movers: Query<(Entity, &Transform), SelectedCarrier>,
    stations: Query<(&Transform, &RangeRing), With<Station>>,
    storages: Query<(Entity, &Transform), With<Storage>>,
    mut hauls: MessageWriter<CargoHaul>,
) {
    let load = keyboard.just_pressed(LOAD_KEY);
    let unload = keyboard.just_pressed(UNLOAD_KEY);
    if !(load || unload) {
        return;
    }

    // Exactly one selected mover, or there's nothing unambiguous to load. Bail rather than panic on
    // 0 or >1 (single-selection means >1 shouldn't happen, but don't assume it).
    let mut it = movers.iter();
    let Some((mover, mover_tf)) = it.next() else {
        return;
    };
    if it.next().is_some() {
        return;
    }
    let mover_pos = mover_tf.translation.xy();

    // The station the mover is parked in: nearest whose radius contains it.
    let Some((station_pos, radius)) = stations
        .iter()
        .map(|(tf, ring)| (tf.translation.xy(), ring.radius))
        .filter(|(pos, radius)| mover_pos.distance(*pos) <= *radius)
        .min_by(|(a, _), (b, _)| mover_pos.distance(*a).total_cmp(&mover_pos.distance(*b)))
    else {
        return;
    };

    // Every warehouse in that station's catchment transacts with the mover.
    for (storage, storage_tf) in &storages {
        if storage_tf.translation.xy().distance(station_pos) <= radius {
            let (source, dest) = if load {
                (storage, mover)
            } else {
                (mover, storage)
            };
            hauls.write(CargoHaul { source, dest });
        }
    }
}
