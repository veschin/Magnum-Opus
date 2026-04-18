//! Building tag component attached by the grid drain when the PlaceTile
//! payload carries a Some(BuildingType).
//!
//! Carries only the type id. F5 will introduce ProductionState and buffer
//! components alongside Building; no field lives here.

use super::types::BuildingType;
use bevy::prelude::Component;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Building {
    pub building_type: BuildingType,
}
