//! Building palette + geometry for the pixel-art pipeline. Each type renders
//! as a flat-shaded cuboid standing on top of its tile; height varies per
//! type so the iso silhouettes are distinguishable.

use crate::buildings::BuildingType;
use bevy::color::{LinearRgba, Srgba};

/// Footprint edge length (XZ) of a building cuboid in world units. Smaller
/// than `TILE_WORLD_SIZE` so the terrain colour reads around the base.
pub const BUILDING_WORLD_SIZE: f32 = 0.7;

fn srgb_u8(r: u8, g: u8, b: u8) -> LinearRgba {
    Srgba::rgb_u8(r, g, b).into()
}

pub fn building_linear(btype: BuildingType) -> LinearRgba {
    match btype {
        BuildingType::Miner => srgb_u8(0xd4, 0xa5, 0x4a),
        BuildingType::Smelter => srgb_u8(0xc8, 0x64, 0x28),
        BuildingType::Mall => srgb_u8(0xa8, 0x50, 0xc8),
        BuildingType::EnergySource => srgb_u8(0x64, 0xd6, 0xff),
    }
}

/// Vertical extent of a building's cuboid. Picked for visual differentiation
/// in iso view; tuning is cosmetic only.
pub fn building_height(btype: BuildingType) -> f32 {
    match btype {
        BuildingType::Miner => 0.5,
        BuildingType::Smelter => 1.0,
        BuildingType::Mall => 1.5,
        BuildingType::EnergySource => 1.2,
    }
}
