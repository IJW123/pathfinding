//! The bake pipeline: scan the sprite dir, turn each PNG's opaque silhouette into a normalized
//! convex hull, and write the manifest.

use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::path::Path;

use crate::constants::{ALPHA_OPAQUE, ASSET_PATH_PREFIX, MANIFEST_PATH, SPRITES_DIR};
use crate::hull::convex_hull;
use crate::manifest::{Manifest, RawSpriteDef};

/// Bake every `*.png` under [`SPRITES_DIR`] into [`MANIFEST_PATH`]. Fails loudly on the first bad
/// asset rather than emitting a partial manifest.
///
/// # Errors
/// I/O failure reading the sprite dir / writing the manifest, an undecodable image, or a degenerate
/// silhouette (fewer than 3 non-collinear opaque points).
pub fn run() -> Result<(), Box<dyn Error>> {
    let mut sprites: BTreeMap<String, RawSpriteDef> = BTreeMap::new();

    for entry in fs::read_dir(SPRITES_DIR)? {
        let path = entry?.path();
        if path.extension().and_then(|e| e.to_str()) != Some("png") {
            continue;
        }
        let id = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| format!("non-UTF8 sprite filename: {}", path.display()))?
            .to_owned();

        let def = bake_one(&path)?;
        println!("baked '{}' -> {} hull points", id, def.hull.len());
        sprites.insert(id, def);
    }

    let manifest = Manifest { sprites };
    let pretty = ron::ser::PrettyConfig::default();
    let text = ron::ser::to_string_pretty(&manifest, pretty)?;
    fs::write(MANIFEST_PATH, text)?;
    println!(
        "wrote {} ({} sprites)",
        MANIFEST_PATH,
        manifest.sprites.len()
    );
    Ok(())
}

/// Bake a single image into its normalized hull def.
fn bake_one(path: &Path) -> Result<RawSpriteDef, Box<dyn Error>> {
    let img = image::open(path)?.to_rgba8();
    let (width, height) = img.dimensions();

    let pixels = opaque_extents(&img);
    let hull = convex_hull(&pixels);
    if hull.len() < 3 {
        return Err(format!(
            "'{}' has a degenerate silhouette ({} hull points): needs >= 3 non-collinear opaque \
             pixels. Check the alpha channel.",
            path.display(),
            hull.len()
        )
        .into());
    }

    let image_path = format!(
        "{ASSET_PATH_PREFIX}/{}",
        path.file_name().and_then(|n| n.to_str()).expect("png name")
    );
    let normalized = normalize(&hull, width, height);
    Ok(RawSpriteDef {
        image_path,
        aspect: width as f32 / height as f32,
        hull: normalized,
    })
}

/// Per-row left/right extremes of opaque pixels. Any opaque pixel sits between its row's min and
/// max x, so these extremes alone contain every convex-hull vertex — far fewer points than the full
/// opaque set, same hull.
fn opaque_extents(img: &image::RgbaImage) -> Vec<(f32, f32)> {
    let (width, height) = img.dimensions();
    let mut points = Vec::new();
    for y in 0..height {
        let mut min_x = None;
        let mut max_x = 0u32;
        for x in 0..width {
            if img.get_pixel(x, y).0[3] >= ALPHA_OPAQUE {
                min_x.get_or_insert(x);
                max_x = x;
            }
        }
        if let Some(min_x) = min_x {
            // Pixel centers; min and max coincide on a 1px-wide row, dedup handles it.
            points.push((min_x as f32 + 0.5, y as f32 + 0.5));
            points.push((max_x as f32 + 0.5, y as f32 + 0.5));
        }
    }
    points
}

/// Map pixel-space hull points into the normalization frame the sprite is drawn in: image-center
/// origin, y-up, divided by the longest image side (so longest side spans 1.0).
fn normalize(hull: &[(f32, f32)], width: u32, height: u32) -> Vec<(f32, f32)> {
    let longest = width.max(height) as f32;
    let half_w = width as f32 / 2.0;
    let half_h = height as f32 / 2.0;
    hull.iter()
        .map(|&(px, py)| ((px - half_w) / longest, (half_h - py) / longest))
        .collect()
}
