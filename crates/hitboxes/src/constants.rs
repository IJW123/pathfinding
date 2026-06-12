/// Hull-validation collinearity threshold on the *sine* of a corner's turn angle
/// (dimensionless, so scale-free). Corners turning less than this count as straight.
pub const HULL_COLLINEAR_EPSILON: f32 = 1e-4;
