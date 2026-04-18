//! Position component attached to every entity placed on the grid.
//!
//! Minimal MVP shape. F4 (buildings) extends placed entities with `Building`,
//! F5 (recipes) adds `Recipe`, etc. Position alone is what grid.occupancy
//! maps to, so every entry in the BTreeMap has a matching Position.

use bevy::prelude::Component;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub x: u32,
    pub y: u32,
}
