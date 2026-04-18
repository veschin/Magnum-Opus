//! F1 world-foundation / AC2: Grid dims populated after first tick, occupancy empty.

use magnum_opus::core::*;
use magnum_opus::grid::{Grid, GridModule};
use magnum_opus::placement::PlacementInputModule;
use magnum_opus::world_config::WorldConfigModule;

#[test]
fn grid_bootstrap_copies_dims_from_world_config() {
    let mut app = Harness::new()
        .with_data::<WorldConfigModule>()
        .with_sim::<GridModule>()
        .with_input::<PlacementInputModule>()
        .build();
    app.update();

    let grid = app.world().resource::<Grid>();
    assert!(grid.dims_set);
    assert_eq!(grid.width, 64);
    assert_eq!(grid.height, 64);
    assert!(grid.occupancy.is_empty());
}
