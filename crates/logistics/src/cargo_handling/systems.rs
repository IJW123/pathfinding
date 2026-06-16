use bevy::prelude::*;

use crate::cargo_handling::components::{Carrier, DockZone};
use crate::cargo_handling::message::CargoHaul;
use crate::commodity::Commodity;
use crate::components::{Capacity, Inventory, Storage};

/// Apply [`CargoHaul`]s: per commodity, move `min(available at source, grantable at dest)`, so the
/// dest never breaches its caps and the unmoved remainder stays in the source. Grantable is
/// recomputed against the *running* dest state, so the dest's shared weight/volume budget is honoured
/// across commodities. Hauls with a missing endpoint, or `source == dest`, are dropped.
pub fn apply_cargo_hauls(
    mut reader: MessageReader<CargoHaul>,
    mut inventories: Query<&mut Inventory>,
    capacities: Query<&Capacity>,
) {
    for haul in reader.read() {
        if haul.source == haul.dest {
            continue;
        }
        let Ok([mut source, mut dest]) = inventories.get_many_mut([haul.source, haul.dest]) else {
            continue;
        };
        let dest_capacity = capacities.get(haul.dest).ok();
        for commodity in Commodity::ALL {
            let available = source.amount(commodity);
            let grant =
                dest_capacity.map_or(available, |c| c.grantable(&dest, commodity, available));
            let moved = source.remove(commodity, grant);
            dest.add(commodity, moved);
        }
    }
}

/// Keyboard docking: `9` loads everything from the nearest in-range storage into the carrier, `0`
/// unloads the carrier back into it. A storage is in range when the carrier sits within its
/// [`DockZone`]. Placeholder input — a real order/UI replaces it later.
pub fn dock_haul_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    carriers: Query<(Entity, &Transform), With<Carrier>>,
    zones: Query<(Entity, &Transform, &DockZone), With<Storage>>,
    mut hauls: MessageWriter<CargoHaul>,
) {
    let load = keyboard.just_pressed(KeyCode::Digit9);
    let unload = keyboard.just_pressed(KeyCode::Digit0);
    if load || unload {
        for (carrier, carrier_tf) in &carriers {
            let here = carrier_tf.translation.xy();
            let nearest = zones
                .iter()
                .map(|(storage, tf, zone)| {
                    (storage, here.distance(tf.translation.xy()), zone.radius)
                })
                .filter(|(_, dist, radius)| dist <= radius)
                .min_by(|a, b| a.1.total_cmp(&b.1));
            if let Some((storage, _, _)) = nearest {
                let haul = if load {
                    CargoHaul {
                        source: storage,
                        dest: carrier,
                    }
                } else {
                    CargoHaul {
                        source: carrier,
                        dest: storage,
                    }
                };
                hauls.write(haul);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    /// Spawn a holder with an inventory and optional capacity, return its id.
    fn holder(world: &mut World, inv: Inventory, cap: Option<Capacity>) -> Entity {
        let mut e = world.spawn(inv);
        if let Some(cap) = cap {
            e.insert(cap);
        }
        e.id()
    }

    fn haul(world: &mut World, source: Entity, dest: Entity) {
        world.init_resource::<Messages<CargoHaul>>();
        world.write_message(CargoHaul { source, dest });
        world
            .run_system_once(apply_cargo_hauls)
            .expect("apply_cargo_hauls runs");
    }

    #[test]
    fn full_move_when_dest_is_roomy() {
        let mut world = World::new();
        let src = holder(
            &mut world,
            Inventory {
                grain: 10,
                ..Inventory::default()
            },
            None,
        );
        let dst = holder(&mut world, Inventory::default(), None);
        haul(&mut world, src, dst);

        assert_eq!(world.get::<Inventory>(src).unwrap().grain, 0);
        assert_eq!(world.get::<Inventory>(dst).unwrap().grain, 10);
    }

    #[test]
    fn partial_move_leaves_remainder_in_source() {
        let mut world = World::new();
        let src = holder(
            &mut world,
            Inventory {
                grain: 10, // 250 kg
                ..Inventory::default()
            },
            None,
        );
        // dest holds only 100 kg → 4 grain fit, 6 stay behind.
        let dst = holder(
            &mut world,
            Inventory::default(),
            Some(Capacity {
                max_weight: Some(100.0),
                max_volume: None,
            }),
        );
        haul(&mut world, src, dst);

        assert_eq!(world.get::<Inventory>(dst).unwrap().grain, 4, "only 4 fit");
        assert_eq!(
            world.get::<Inventory>(src).unwrap().grain,
            6,
            "remainder stays in source"
        );
    }

    #[test]
    fn shared_budget_across_commodities() {
        let mut world = World::new();
        // 10 grain (250 kg) + 10 coal (300 kg) at source.
        let src = holder(
            &mut world,
            Inventory {
                grain: 10,
                coal: 10,
                ..Inventory::default()
            },
            None,
        );
        // 300 kg cap: grain goes first (ALL order) → 10 grain = 250 kg, then 1 coal = 30 kg (280),
        // a 2nd coal would be 310 > 300, so coal stops at 1.
        let dst = holder(
            &mut world,
            Inventory::default(),
            Some(Capacity {
                max_weight: Some(300.0),
                max_volume: None,
            }),
        );
        haul(&mut world, src, dst);

        let got = world.get::<Inventory>(dst).unwrap();
        assert_eq!(got.grain, 10);
        assert_eq!(got.coal, 1);
    }

    #[test]
    fn source_equals_dest_is_noop() {
        let mut world = World::new();
        let e = holder(
            &mut world,
            Inventory {
                grain: 5,
                ..Inventory::default()
            },
            None,
        );
        haul(&mut world, e, e);
        assert_eq!(world.get::<Inventory>(e).unwrap().grain, 5);
    }
}
