use bevy::prelude::*;

/// Tags the outline child spawned under a selected entity, so deselection can find and despawn it.
#[derive(Component)]
pub struct HighlightMarker;
