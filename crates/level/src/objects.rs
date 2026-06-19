//! The authored level and the composition that spawns it. [`spec::LevelSpec`] is the whole-level
//! root — map size, terrain recipe, and every object — loaded from `assets/level.ron` ([`loader`])
//! via the raw serde [`manifest`] types, then mapped to bundles by the per-object constructors
//! ([`storage`], [`player`]). `level` is the composition root, so cross-crate concerns
//! (selection/motion/sprites) compose here without any domain crate learning about them. Mirrors the
//! in-repo `sprites` pipeline (raw manifest → in-memory resource → consumed at spawn).

pub mod constants;
pub mod loader;
pub mod manifest;
pub mod player;
pub mod spec;
pub mod storage;
