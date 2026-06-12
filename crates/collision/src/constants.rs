pub const CELL_SIZE: f32 = 80.0;

/// Collision-wide tolerance. Double duty: a length/normalize guard and the touching→`None`
/// penetration gate. Both want `1e-4` today; split into two named constants only if the scales
/// ever need to diverge.
pub const COLLISION_EPSILON: f32 = 1e-4;

/// Max Gauss-Seidel passes over the solid pair list per tick. The 50/50 dynamic split halves
/// chain residuals per pass (ratio 0.5), so a 5u-deep chain settles under slop in ~9 passes;
/// 12 is headroom. Early-exit stops typical ticks after 1–3 passes.
pub const SOLVER_ITERATIONS: usize = 12;

/// Penetration left unresolved on purpose (world units; 1.25% of the 40u player — invisible).
/// Gates corrections only, never detection, so resting contacts keep emitting events without
/// being pushed around (rest-jitter kill).
pub const PENETRATION_SLOP: f32 = 0.5;

/// Fraction of (depth − slop) corrected per pass. Full projection: a purely positional solver
/// can't overshoot a single contact (it lands exactly on contact+slop), slop handles rest
/// jitter, and under-relaxation would only slow chain convergence. Kept as a knob for when
/// mass/softness arrive.
pub const PENETRATION_PERCENT: f32 = 1.0;

/// Broad-phase AABB inflation (world units). Must exceed the largest displacement a body can
/// accrue mid-solve, bounded by the max per-tick step: 300 u/s ÷ 64 Hz ≈ 4.7u. 8.0 ≈ 70%
/// headroom, so pairs created by other corrections are already on the list.
pub const BROAD_PHASE_MARGIN: f32 = 8.0;
