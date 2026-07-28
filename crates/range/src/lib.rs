//! General range components.
//!
//! A [`RangeRing`](components::RangeRing) is a world-logic declaration that an entity has a circular
//! range of a given radius. The radius is gameplay data and lives here; how (or whether) it's drawn
//! is the renderer's call. Attach it to anything that should expose a range — a station's catchment
//! today, other range types later.

pub mod components;
