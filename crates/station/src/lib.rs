//! Rail stations: cargo broker nodes that load and unload anything passing through.
//!
//! A [`Station`](components::Station) holds no goods of its own. When a [`Carrier`] (the player
//! avatar or a rail train) is parked inside a station's radius, a load/unload key brokers a cargo
//! transfer between that mover and every warehouse ([`Storage`]) in the same radius. The radius is a
//! [`RangeRing`] (world-logic data the renderer also draws), and the transfer rides logistics'
//! existing `CargoHaul`/`apply_cargo_hauls` engine — this crate only adds the station node and the
//! mover-side trigger, bridging movers and warehouses without coupling `rail` and `logistics`.
//!
//! [`Carrier`]: logistics::cargo_handling::components::Carrier
//! [`Storage`]: logistics::components::Storage
//! [`RangeRing`]: range::components::RangeRing

pub mod bundle;
pub mod components;
pub mod constants;
pub mod plugin;
pub mod systems;
