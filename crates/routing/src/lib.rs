//! Grade-constrained terrain routing.
//!
//! Generic grid A* that connects two world points with a polyline whose every step stays under a
//! caller-supplied maximum **grade** (elevation delta over horizontal distance). The same algorithm
//! serves rail (strict grade), roads (looser grade), and off-road agents — they differ only in the
//! [`PathProfile`] they pass.
//!
//! Elevation is read through the [`ElevationSampler`] trait, so this crate stays a pure, terrain-
//! agnostic leaf: it depends on no game crate and is testable with mock fields.

mod astar;
mod constants;
mod grid;
mod profile;
mod sampler;
mod simplify;

pub use astar::find_path;
pub use profile::PathProfile;
pub use sampler::ElevationSampler;
