//! Derived world measurements.
//!
//! Read-only observations computed from world state after everyone else has mutated it. Each
//! measurement is its own module: [`motion`] observes per-frame `Transform` deltas as velocity;
//! future measurements (heading, throughput, dwell-time, …) join as sibling modules. World-logic
//! only — rendering reads these components but never depends the other way.

pub mod motion;
