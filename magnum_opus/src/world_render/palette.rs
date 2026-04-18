//! Palette + per-kind geometry for the pixel-art scene.
//!
//! Every terrain kind renders as a 1×1 footprint cuboid; vertical size
//! differentiates silhouettes (mountains loom, pits sink). Colours are
//! returned as `LinearRgba` because the `ToonMaterial` uniform expects
//! linear-space inputs and the shader multiplies against a banded `NdotL`
//! term directly. sRGB byte literals are kept in comments so they can be
//! eyeballed against art references.

use crate::landscape::{TerrainCell, TerrainKind};
use crate::resources::ResourceKind;
use bevy::color::{LinearRgba, Srgba};
use bevy::prelude::Vec3;

/// XZ footprint of a tile in world units.
pub const TILE_WORLD_SIZE: f32 = 1.0;

/// Half the grid extent - the 64×64 grid is centred on origin.
pub const GRID_HALF: f32 = 32.0;

/// Sphere radius used for vein markers sitting on top of their tile.
pub const VEIN_RADIUS: f32 = 0.22;

fn srgb_u8(r: u8, g: u8, b: u8) -> LinearRgba {
    Srgba::rgb_u8(r, g, b).into()
}

pub fn terrain_linear(kind: TerrainKind) -> LinearRgba {
    match kind {
        // Grass #7dc858 - saturated green for the dominant tile.
        TerrainKind::Grass => srgb_u8(0x7d, 0xc8, 0x58),
        // Rock #8a8a8a - mid grey.
        TerrainKind::Rock => srgb_u8(0x8a, 0x8a, 0x8a),
        // Water #3a6db5 - deep blue.
        TerrainKind::Water => srgb_u8(0x3a, 0x6d, 0xb5),
        // Lava #ff5522 - hot orange.
        TerrainKind::Lava => srgb_u8(0xff, 0x55, 0x22),
        // Sand #e8c878 - warm beige.
        TerrainKind::Sand => srgb_u8(0xe8, 0xc8, 0x78),
        // Mountain #a8a8b4 - cold lit grey, slightly bluer than rock.
        TerrainKind::Mountain => srgb_u8(0xa8, 0xa8, 0xb4),
        // Pit #181824 - near-black.
        TerrainKind::Pit => srgb_u8(0x18, 0x18, 0x24),
    }
}

pub fn resource_linear(kind: ResourceKind) -> LinearRgba {
    match kind {
        ResourceKind::IronOre => srgb_u8(0xc0, 0x60, 0x40),
        ResourceKind::CopperOre => srgb_u8(0xd8, 0x70, 0x38),
        ResourceKind::Stone => srgb_u8(0xbb, 0xbb, 0xbb),
        ResourceKind::Coal => srgb_u8(0x1a, 0x1a, 0x1a),
    }
}

/// Vertical extent of a terrain cuboid. Ground plane is at `y = 0`; tiles sit
/// with base at `y = 0` for land, sunken below for water / pit.
pub fn terrain_height(kind: TerrainKind) -> f32 {
    match kind {
        TerrainKind::Grass => 0.40,
        TerrainKind::Sand => 0.35,
        TerrainKind::Lava => 0.30,
        TerrainKind::Water => 0.15,
        TerrainKind::Pit => 0.10,
        TerrainKind::Rock => 1.20,
        TerrainKind::Mountain => 3.00,
    }
}

/// Per-cell top offset derived from the cell's elevation. Adds a small
/// variation (±~0.35 world units) on top of the terrain kind height so that
/// neighbouring tiles of the same kind land at slightly different y levels.
/// Without this, vast same-kind patches collapse into a single flat plane
/// with all side faces hidden under each tile's neighbour, and the iso
/// projection reads as a coloured diagram instead of a 3D scene.
pub fn cell_elevation_offset(elevation: i8) -> f32 {
    const MAX_OFFSET: f32 = 0.35;
    let t = (elevation as f32) / 64.0;
    t.clamp(-1.0, 1.0) * MAX_OFFSET
}

pub fn cell_top_height(cell: TerrainCell) -> f32 {
    (terrain_height(cell.kind) + cell_elevation_offset(cell.elevation)).max(0.05)
}

/// Y offset of a tile's base - land starts at 0, water and pit dip below.
pub fn terrain_base_y(kind: TerrainKind) -> f32 {
    match kind {
        TerrainKind::Water => -0.15,
        TerrainKind::Pit => -0.35,
        _ => 0.0,
    }
}

/// Top surface Y of a tile - where buildings / veins sit. Takes the whole
/// `TerrainCell` so the per-cell elevation offset is included; without it,
/// things stacked on top (buildings, veins) float above or clip into the
/// jittered tile top.
pub fn terrain_top_y(cell: TerrainCell) -> f32 {
    terrain_base_y(cell.kind) + cell_top_height(cell)
}

/// World-space centre of tile (x, y) on the XZ ground plane. Y is computed so
/// the cuboid base sits at `terrain_base_y(kind)` with a height derived from
/// `cell_top_height(cell)` (kind height + elevation jitter).
pub fn tile_world_pos(x: u32, y: u32, cell: TerrainCell) -> Vec3 {
    let h = cell_top_height(cell);
    let base = terrain_base_y(cell.kind);
    Vec3::new(
        x as f32 - GRID_HALF + 0.5,
        base + h / 2.0,
        y as f32 - GRID_HALF + 0.5,
    )
}

/// World-space XZ centre of tile (x, y), for placing objects on top of tiles
/// when you only need the grid coordinate (not the terrain-height offset).
pub fn tile_center_xz(x: u32, y: u32) -> (f32, f32) {
    (x as f32 - GRID_HALF + 0.5, y as f32 - GRID_HALF + 0.5)
}
