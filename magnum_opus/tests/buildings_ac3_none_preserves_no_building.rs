//! F4 AC3 - placement with None leaves the entity without a Building component.
//! Preserves F3 behaviour as backwards compatibility.

use magnum_opus::buildings::{Building, BuildingDbModule};
use magnum_opus::core::{AppExt, CommandBus, Harness};
use magnum_opus::grid::{Grid, GridModule, PlaceTile, Position};
use magnum_opus::placement::PlacementInputModule;
use magnum_opus::world_config::WorldConfigModule;

#[test]
fn untyped_placement_does_not_attach_building_component() {
    let mut app = Harness::new()
        .with_data::<WorldConfigModule>()
        .with_data::<BuildingDbModule>()
        .with_sim::<GridModule>()
        .with_input::<PlacementInputModule>()
        .build();

    app.world_mut()
        .resource_mut::<CommandBus<PlaceTile>>()
        .push(PlaceTile {
            x: 6,
            y: 6,
            building_type: None,
        });

    app.update();
    app.update();

    let grid = app.world().resource::<Grid>();
    let entity = *grid.occupancy.get(&(6, 6)).expect("(6,6) must be occupied");

    assert!(app.world().get::<Position>(entity).is_some());
    assert!(
        app.world().get::<Building>(entity).is_none(),
        "untyped PlaceTile must not attach Building component"
    );
}
