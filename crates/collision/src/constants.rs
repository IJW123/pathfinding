pub const CELL_SIZE: f32 = 80.0;

/// Collision-wide tolerance. Double duty: a length/normalize guard and the touching→`None`
/// penetration gate. Both want `1e-4` today; split into two named constants only if the scales
/// ever need to diverge.
pub const COLLISION_EPSILON: f32 = 1e-4;

/// Hull-validation collinearity threshold on the *sine* of a corner's turn angle
/// (dimensionless, so scale-free). Corners turning less than this count as straight.
pub const HULL_COLLINEAR_EPSILON: f32 = 1e-4;
