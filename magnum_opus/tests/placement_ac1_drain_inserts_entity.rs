//! F3 AC1 - happy-path placement drain.
//!
//! Pushing a PlaceTile for a valid cell and ticking twice must result in a
//! spawned entity carrying a matching Position component, indexed by the
//! (x, y) coordinate inside Grid.occupancy.

use magnum_opus::core::{CommandBus, Harness};
use magnum_opus::grid::{Grid, GridModule, PlaceTile, Position};
use magnum_opus::placement::PlacementInputModule;
use magnum_opus::world_config::WorldConfigModule;

#[test]
fn placement_drain_inserts_entity_with_position() {
    let mut app = Harness::new()
        .with_data::<WorldConfigModule>()
        .with_sim::<GridModule>()
        .with_input::<PlacementInputModule>()
        .build();

    app.world_mut()
        .resource_mut::<CommandBus<PlaceTile>>()
        .push(PlaceTile {
            x: 3,
            y: 4,
            ..Default::default()
        });

    app.update();
    app.update();

    let grid = app.world().resource::<Grid>();
    assert_eq!(grid.occupancy.len(), 1, "one tile expected in occupancy");
    let entity = *grid
        .occupancy
        .get(&(3, 4))
        .expect("occupancy must contain (3, 4)");

    let pos = app
        .world()
        .get::<Position>(entity)
        .expect("spawned entity must carry a Position component");
    assert_eq!(pos, &Position { x: 3, y: 4 });
}
