use bevy::prelude::Component;

// `Default + Clone` make each marker its own BSN template, so it can be named bare in `bsn!`.

/// The text readout (position, elevation, speed) in the top-right corner.
#[derive(Component, Default, Clone)]
pub struct HudReadout;

/// The filled bar of the scale widget; its width (logical px) is set each frame
/// to represent the labelled "nice" distance at the current zoom.
#[derive(Component, Default, Clone)]
pub struct ScaleBarFill;

/// The text label beneath the scale bar (e.g. `"200 m"`, `"1 km"`).
#[derive(Component, Default, Clone)]
pub struct ScaleBarLabel;
