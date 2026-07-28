use bevy::prelude::*;

/// Marker for a mobile cargo holder that loads/unloads at a station in range — the player and rail
/// trains today, other vessels later. The station broker finds movers through this marker, so
/// `logistics` needn't depend on `rail` or `player`.
#[derive(Component)]
pub struct Carrier;
