use bevy::prelude::*;

use crate::systems::station_haul_input;

/// Wires the station cargo-broker trigger. It writes [`CargoHaul`](logistics) messages but does
/// **not** register them or the transfer system — it relies on `LogisticsPlugin` for
/// `add_message::<CargoHaul>` + `apply_cargo_hauls`, so add this alongside `LogisticsPlugin`.
///
/// No cross-plugin ordering vs `apply_cargo_hauls`: Bevy messages live a frame, so a same-frame miss
/// just applies on the next tick — harmless for cargo, and not worth a shared `SystemSet`.
pub struct StationPlugin;

impl Plugin for StationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            station_haul_input.run_if(resource_exists::<ButtonInput<KeyCode>>),
        );
    }
}
