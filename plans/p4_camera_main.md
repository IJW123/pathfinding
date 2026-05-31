# Plan 4: `camera_main` crate — WASD camera scrolling

# Goal

Add WASD camera panning via a new dedicated `camera_main` crate. Moving the camera
is the whole feature: chunk streaming (`world::elevation::chunk_lifecycle::update_loaded_chunks`)
already queries `Single<&Transform, With<Camera2d>>` and loads/unloads chunks around
`cam_pos`, so it follows the camera for free.

# Status: 📝 PLANNED

# Context (verified against current code)

- Camera spawned in `crates/app/src/main.rs:28` as a bare `Camera2d` (no marker, default Transform at origin), never moves.
- Player movement uses **arrow keys** (`crates/player/src/systems.rs:31`) → **WASD is free**, no input conflict.
- Chunk streaming keys off `With<Camera2d>` generically (`crates/world/src/elevation/chunk_lifecycle.rs:15`) → **no marker component needed**, so the new crate needs no reverse dependency.
- Crate convention: `lib.rs` = `pub mod` decls only; `plugin.rs` = the `Plugin`; `systems.rs` = systems; `constants.rs` = consts. Deps go in root `[workspace.dependencies]`, pulled with `{ workspace = true }`.

# Bevy 0.18 APIs (confirmed in live codebase)

- `time.delta_secs()` — `player/systems.rs:49`, `motion/systems.rs:30`
- `Single<&mut Transform, With<Camera2d>>` — `chunk_lifecycle.rs:15`
- `commands.spawn(Camera2d)` — `app/main.rs:29`
- `keyboard.pressed(KeyCode::KeyW)` — `player/systems.rs:31`
- `add_systems(Startup/Update, ...)` — `player/plugin.rs`
- `Vec2::normalize_or_zero()`, `Vec2::extend(0.0)` — stdlib of `bevy::math`

# Steps

## 1. New crate `crates/camera_main/`

**`Cargo.toml`**
```toml
[package]
name = "camera_main"
version.workspace = true
edition.workspace = true

[dependencies]
bevy = { workspace = true }
```

**`src/lib.rs`** (module decls only — no items in lib root)
```rust
pub mod constants;
pub mod plugin;
pub mod systems;
```

**`src/constants.rs`**
```rust
pub const CAMERA_PAN_SPEED: f32 = 500.0;
```

**`src/systems.rs`**
```rust
use bevy::prelude::*;

use crate::constants::CAMERA_PAN_SPEED;

pub fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

pub fn pan_camera(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut camera: Single<&mut Transform, With<Camera2d>>,
) {
    let mut direction = Vec2::ZERO;
    if keyboard.pressed(KeyCode::KeyW) {
        direction.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        direction.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }

    let delta = direction.normalize_or_zero() * CAMERA_PAN_SPEED * time.delta_secs();
    camera.translation += delta.extend(0.0);
}
```

**`src/plugin.rs`**
```rust
use bevy::prelude::*;

use crate::systems::{pan_camera, spawn_camera};

pub struct CameraMainPlugin;

impl Plugin for CameraMainPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera)
            .add_systems(Update, pan_camera);
    }
}
```

## 2. Wire into workspace

- Root `Cargo.toml`:
  - add `"crates/camera_main"` to `members`
  - add `camera_main = { path = "crates/camera_main" }` to `[workspace.dependencies]`
- `crates/app/Cargo.toml`: add `camera_main = { workspace = true }`

## 3. Update `crates/app/src/main.rs`

- Delete `fn setup_camera` and the `.add_systems(Startup, setup_camera)` line.
- Add `use camera_main::plugin::CameraMainPlugin;`
- Add `CameraMainPlugin` to the `add_plugins((...))` tuple.

## 4. Verify

- `cargo build` — clean, zero warnings.
- Run: WASD pans the view; chunks stream in/out around the moving camera; arrow keys still move the player independently.
- Kill the app afterwards.

# Decisions (made, not open)

- **No `MainCamera` marker** — chunk_lifecycle already keys off `Camera2d`; a marker would force `world` to depend on `camera_main`. Skip it.
- **No system-set ordering** — `pan_camera` and `update_loaded_chunks` both in `Update`; one-frame lag on chunk reaction is invisible.
- **Diagonal normalized** (`normalize_or_zero`) so diagonal pan isn't faster; frame-rate independent via `delta_secs()`.

# Risk note

Panning now exercises the chunk load/unload path continuously, where a static camera
never did. Per the known elevation lifecycle fragilities, this is the place to look
if chunks flicker/leak during fast scroll. Panning won't *introduce* those bugs, just
surface them.
