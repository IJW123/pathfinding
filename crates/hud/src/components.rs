use bevy::prelude::Component;

/// The text readout (position, elevation, speed) in the top-right corner.
#[derive(Component)]
pub struct HudReadout;

/// The filled bar of the scale widget; its width (logical px) is set each frame
/// to represent the labelled "nice" distance at the current zoom.
#[derive(Component)]
pub struct ScaleBarFill;

/// The text label beneath the scale bar (e.g. `"200 m"`, `"1 km"`).
#[derive(Component)]
pub struct ScaleBarLabel;
