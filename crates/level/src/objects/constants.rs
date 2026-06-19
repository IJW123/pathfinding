//! Where the authored level lives. The object magnitudes that used to sit here (capacities, dock
//! radius, sizes) now live in the asset itself — see `assets/level.ron`.

/// Authored level layout, relative to the working directory (workspace root under `cargo run`).
/// Matches the path the loader reads; mirrors `sprites`' `MANIFEST_PATH` convention.
pub const LEVEL_PATH: &str = "assets/level.ron";
