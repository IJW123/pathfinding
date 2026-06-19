//! Composition magnitudes for the world-object constructors (logistics owns the model; level sets
//! the magnitudes). Sizes that are level *layout* knobs (e.g. `STORAGE_HALF_EXTENT`) stay in
//! `level::constants` — these are the capacity/docking numbers the constructors bake in.

/// Storage holds by space only (it's on the ground); m³. Above its starting stock so the seed is
/// valid but deposits still cap.
pub const STORAGE_MAX_VOLUME: f32 = 20.0;
/// Circular load/unload range around the storage (world units), drawn around its ~50-unit body.
pub const STORAGE_DOCK_RADIUS: f32 = 120.0;
/// The player carries cargo: capped on both axes so hauling a full building clamps (shows partial
/// fill). Weight in kg, volume in m³.
pub const CARRIER_MAX_WEIGHT: f32 = 2000.0;
pub const CARRIER_MAX_VOLUME: f32 = 3.0;
