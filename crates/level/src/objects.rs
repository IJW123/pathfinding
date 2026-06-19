//! Per-object entity composition: "what an X entity is made of" lives here, one constructor per
//! object, each returning the complete bundle. `spawn::spawn_level` is then pure layout (positions
//! and sizes) — it just spawns these. `level` is the composition root, so cross-crate concerns
//! (selection/motion/sprites) compose here without any domain crate learning about them.

pub mod constants;
pub mod player;
pub mod storage;
