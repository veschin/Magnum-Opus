//! F3 AC3 - pushing two PlaceTile onto the same cell keeps the first entity.
//!
//! First command succeeds. Second is rejected because the cell is occupied;
//! it does not overwrite the existing entity.

use magnum_opus::core::{CommandBus, Harness};
use magnum_opus::grid::{Grid, GridModule, PlaceTile};
use magnum_opus::placement::PlacementInputModule;
use magnum_opus::world_config::WorldConfigModule;

#[test]
fn occupied_tile_rejects_subsequent_placement() {
    let mut app = Harness::new()
        .with_data::<WorldConfigModule>()
        .with_sim::<GridModule>()
        .with_input::<PlacementInputModule>()
        .build();

    {
        let mut bus = app.world_mut().resource_mut::<CommandBus<PlaceTile>>();
        bus.push(PlaceTile { x: 7, y: 7 });
        bus.push(PlaceTile { x: 7, y: 7 });
    }

    app.update();
    app.update();

    let grid = app.world().resource::<Grid>();
    assert_eq!(
        grid.occupancy.len(),
        1,
        "only the first PlaceTile for an occupied cell should stick"
    );
    assert!(grid.occupancy.contains_key(&(7, 7)));
}
