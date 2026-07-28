use bevy::input::keyboard::KeyCode;

/// Load the selected train from the warehouses in its station's radius.
pub const LOAD_KEY: KeyCode = KeyCode::KeyL;
/// Unload the selected train into the warehouses in its station's radius.
pub const UNLOAD_KEY: KeyCode = KeyCode::KeyU;

/// Catchment radius of a derived station (world units): a rail terminal with a warehouse within this
/// reach earns a station, and a train parked there transacts with the warehouses inside it.
pub const STATION_RADIUS: f32 = 200.0;

/// Z layer for a station's platform: above the rail line (`TRACK_Z` 0.2), below storage buildings and
/// the loco — a world-side ordering policy intrinsic to the kind, like rail's `TRACK_Z`/`LOCO_Z`.
pub const STATION_Z: f32 = 0.3;

/// Cargo-train hold limit by weight (kg). Far above the carrier's so a train hauls in bulk.
pub const TRAIN_MAX_WEIGHT: f32 = 50_000.0;
/// Cargo-train hold limit by volume (m³).
pub const TRAIN_MAX_VOLUME: f32 = 120.0;
