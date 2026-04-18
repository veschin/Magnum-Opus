//! F1 world-foundation / edge case: Grid::default() before first tick is zero-sized.

use magnum_opus::grid::Grid;

#[test]
fn grid_default_reports_unset_dims() {
    let grid = Grid::default();
    assert!(!grid.dims_set);
    assert_eq!(grid.width, 0);
    assert_eq!(grid.height, 0);
    assert!(grid.occupancy.is_empty());
}
