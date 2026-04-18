//! F1 world-foundation / AC6 + AC7: Grid.occupancy is BTreeMap<(u32,u32), Entity>.
//!
//! Compile-time assertion via a typed binding. If the field type drifts
//! (e.g. HashMap, or (i32,i32) key), this file stops compiling.

use bevy::prelude::Entity;
use magnum_opus::grid::Grid;
use std::collections::BTreeMap;

#[test]
fn grid_occupancy_is_btreemap_with_u32_coords() {
    let grid = Grid::default();
    let _typed: &BTreeMap<(u32, u32), Entity> = &grid.occupancy;
    assert!(grid.occupancy.is_empty());
}
