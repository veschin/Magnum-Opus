//! Placement command payload drained by the grid module in `Phase::Commands`.
//!
//! `PlaceTile` carries the target grid coordinate and an optional building
//! type. The grid drain validates bounds + occupancy and spawns a fresh
//! entity with a `Position` component; when `building_type` is Some, the
//! drain also attaches a `Building` component from the buildings module.

use crate::buildings::BuildingType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PlaceTile {
    pub x: u32,
    pub y: u32,
    pub building_type: Option<BuildingType>,
}
