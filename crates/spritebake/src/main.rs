//! Offline sprite-hull bake tool. Run from the workspace root: `cargo run -p spritebake`.
//! Reads every PNG under `assets/sprites/`, approximates each silhouette with a convex hull, and
//! writes `assets/sprite_manifest.ron` for the runtime to load.

use std::process::ExitCode;

mod bake;
mod constants;
mod hull;
mod manifest;

fn main() -> ExitCode {
    match bake::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("spritebake failed: {err}");
            ExitCode::FAILURE
        }
    }
}
