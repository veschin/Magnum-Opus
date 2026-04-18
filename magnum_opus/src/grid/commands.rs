//! Placement command payload drained by the grid module in `Phase::Commands`.
//!
//! `PlaceTile` carries the target grid coordinate. The grid drain validates
//! bounds + occupancy and spawns a bare entity with a `Position` component.
//! No Entity field: the drain always spawns a fresh entity.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlaceTile {
    pub x: u32,
    pub y: u32,
}
