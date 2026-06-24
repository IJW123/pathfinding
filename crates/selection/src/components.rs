use bevy::prelude::*;

/// Marker for entities the cursor can pick (storage, the player, …).
#[derive(Component, Default)]
pub struct Selectable;

/// Marker for the currently controlled entity. Single-selection: the picker keeps at most one
/// entity tagged at a time.
#[derive(Component)]
pub struct Selected;

/// Opt-out from the generic arrow-key free-mover ([`move_selected`](crate::systems::move_selected)).
/// An entity can still be `Selected` (for readouts, picking) but is driven by a domain-specific
/// controller instead — e.g. a rail locomotive constrained to its track. Lives here, not in the
/// owning crate, so `selection` filters on it without depending on that crate (one-way dep graph).
#[derive(Component, Default)]
pub struct FreeMoveExempt;
