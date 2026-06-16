/// Z layer for storage buildings: same band as obstacles — above contour lines (0.1), below the
/// player (1.0). Render-ordering policy intrinsic to the kind; instance positions and size live in
/// the `level` crate.
pub const STORAGE_Z: f32 = 0.5;

// --- Commodity physical properties (per unit weight in kg, density in kg/m³) ---
// Tunable game knobs. Unit volume is derived as weight / density, so these together set how much a
// given stock weighs and how much space it takes up.
pub const GRAIN_UNIT_WEIGHT: f32 = 25.0;
pub const GRAIN_DENSITY: f32 = 770.0;

pub const COAL_UNIT_WEIGHT: f32 = 30.0;
pub const COAL_DENSITY: f32 = 1350.0;

pub const LUMBER_UNIT_WEIGHT: f32 = 20.0;
pub const LUMBER_DENSITY: f32 = 500.0;

pub const IRON_ORE_UNIT_WEIGHT: f32 = 50.0;
pub const IRON_ORE_DENSITY: f32 = 2500.0;
