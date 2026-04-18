//! Building sprite palette and geometry. Copied tile_world_pos from
//! world_render to avoid cross-module coupling; the shared geometry constants
//! should stay in sync.

use crate::buildings::BuildingType;
use bevy::prelude::{Color, Vec3};

pub const TILE_PX: f32 = 4.0;
pub const BUILDING_PX: f32 = 3.0;
const GRID_HALF: f32 = 128.0;

pub fn building_color(btype: BuildingType) -> Color {
    match btype {
        BuildingType::Miner => Color::srgb_u8(0xd4, 0xa5, 0x4a),
        BuildingType::Smelter => Color::srgb_u8(0xc8, 0x64, 0x28),
        BuildingType::Mall => Color::srgb_u8(0xa8, 0x50, 0xc8),
        BuildingType::EnergySource => Color::srgb_u8(0x64, 0xd6, 0xff),
    }
}

pub fn tile_world_pos(x: u32, y: u32) -> Vec3 {
    Vec3::new(
        x as f32 * TILE_PX - GRID_HALF + TILE_PX / 2.0,
        GRID_HALF - TILE_PX / 2.0 - y as f32 * TILE_PX,
        0.0,
    )
}
