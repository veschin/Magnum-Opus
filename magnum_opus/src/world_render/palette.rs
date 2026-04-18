//! Flat-color palette for terrain and resource rendering. MVP placeholder
//! until impostor sprites (albedo + normal + depth) land with F20.

use bevy::prelude::{Color, Vec3};
use crate::landscape::TerrainKind;
use crate::resources::ResourceKind;

pub const TILE_PX: f32 = 4.0;
pub const VEIN_PX: f32 = 2.0;

/// Grid origin offset so tile (0,0) sits at world (-126, 126).
/// 64 tiles * 4 px = 256 px wide; center = 128 px from each edge.
const GRID_HALF: f32 = 128.0;

pub fn terrain_color(kind: TerrainKind) -> Color {
    match kind {
        TerrainKind::Grass => Color::srgb_u8(0x4a, 0x7b, 0x2c),
        TerrainKind::Rock => Color::srgb_u8(0x6c, 0x6c, 0x6c),
        TerrainKind::Water => Color::srgb_u8(0x2c, 0x4e, 0x7b),
        TerrainKind::Lava => Color::srgb_u8(0xc8, 0x4a, 0x1e),
        TerrainKind::Sand => Color::srgb_u8(0xd4, 0xb8, 0x78),
        TerrainKind::Mountain => Color::srgb_u8(0x9c, 0x9c, 0x9c),
        TerrainKind::Pit => Color::srgb_u8(0x1c, 0x1c, 0x1c),
    }
}

pub fn resource_color(kind: ResourceKind) -> Color {
    match kind {
        ResourceKind::IronOre => Color::srgb_u8(0xc8, 0x78, 0x58),
        ResourceKind::CopperOre => Color::srgb_u8(0xb8, 0x78, 0x40),
        ResourceKind::Stone => Color::srgb_u8(0xb0, 0xb0, 0xb0),
        ResourceKind::Coal => Color::srgb_u8(0x28, 0x28, 0x28),
    }
}

pub fn tile_world_pos(x: u32, y: u32) -> Vec3 {
    Vec3::new(
        x as f32 * TILE_PX - GRID_HALF + TILE_PX / 2.0,
        GRID_HALF - TILE_PX / 2.0 - y as f32 * TILE_PX,
        0.0,
    )
}
