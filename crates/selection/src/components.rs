use bevy::prelude::*;

/// Marker for entities the cursor can pick (storage, the player, …).
#[derive(Component)]
pub struct Selectable;

/// Marker for the currently controlled entity. Single-selection: the picker keeps at most one
/// entity tagged at a time.
#[derive(Component)]
pub struct Selected;
