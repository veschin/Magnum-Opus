//! F3 AC5 - CommandBus<PlaceTile> is registered by the grid module installer.
//!
//! Registering GridModule must cause `CommandBus<PlaceTile>` to exist as a
//! Resource, even if no command has been pushed. This enables tests and
//! future InputUI modules to look up the bus without first producing a
//! command.

use magnum_opus::core::{CommandBus, Harness};
use magnum_opus::grid::{GridModule, PlaceTile};
use magnum_opus::placement::PlacementInputModule;
use magnum_opus::world_config::WorldConfigModule;

#[test]
fn place_tile_bus_is_initialized_with_grid_module() {
    let app = Harness::new()
        .with_data::<WorldConfigModule>()
        .with_sim::<GridModule>()
        .with_input::<PlacementInputModule>()
        .build();

    let bus = app
        .world()
        .get_resource::<CommandBus<PlaceTile>>()
        .expect("grid module must init CommandBus<PlaceTile>");
    assert!(bus.is_empty());
}
