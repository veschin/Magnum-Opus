//! F3 AC2 - out-of-bounds placement is silently dropped.
//!
//! Grid is 64x64. A PlaceTile at (100, 100) must not spawn any entity and
//! must not insert anything into occupancy. No panic.

use magnum_opus::core::{CommandBus, Harness};
use magnum_opus::grid::{Grid, GridModule, PlaceTile};
use magnum_opus::placement::PlacementInputModule;
use magnum_opus::world_config::WorldConfigModule;

#[test]
fn out_of_bounds_placement_is_dropped() {
    let mut app = Harness::new()
        .with_data::<WorldConfigModule>()
        .with_sim::<GridModule>()
        .with_input::<PlacementInputModule>()
        .build();

    app.world_mut()
        .resource_mut::<CommandBus<PlaceTile>>()
        .push(PlaceTile { x: 100, y: 100 });

    app.update();
    app.update();

    let grid = app.world().resource::<Grid>();
    assert!(grid.occupancy.is_empty(), "occupancy must remain empty");
}
