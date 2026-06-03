use bevy::prelude::Component;

/// A static contour-line tile covering one chunk of the map. Built once at startup from
/// the immutable `HeightField`; Bevy frustum-culls offscreen tiles. Tags terrain geometry
/// for identification (no coord map — localized re-extraction for future terrain edits
/// would add that).
#[derive(Component)]
pub struct ContourTile;
