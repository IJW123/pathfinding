use bevy::prelude::*;

/// Marker for a mobile cargo holder — the player now, a dedicated vessel later. Cargo systems find
/// the hauler through this, so `logistics` needn't depend on `player`.
#[derive(Component)]
pub struct Carrier;

/// A circular load/unload range around a holder, centred on its `Transform`. A [`Carrier`] within
/// `radius` may haul to or from it. Set per-instance by the spawner (`level`), like size.
#[derive(Component, Debug, Clone, Copy)]
pub struct DockZone {
    pub radius: f32,
}
