//! F4 AC2 - placement with Some(BuildingType) spawns entity with both Position and Building.

use magnum_opus::buildings::{Building, BuildingDbModule, BuildingType};
use magnum_opus::core::{AppExt, CommandBus, Harness};
use magnum_opus::grid::{Grid, GridModule, PlaceTile, Position};
use magnum_opus::placement::PlacementInputModule;
use magnum_opus::world_config::WorldConfigModule;

#[test]
fn typed_placement_spawns_building_entity() {
    let mut app = Harness::new()
        .with_data::<WorldConfigModule>()
        .with_data::<BuildingDbModule>()
        .with_sim::<GridModule>()
        .with_input::<PlacementInputModule>()
        .build();

    app.world_mut()
        .resource_mut::<CommandBus<PlaceTile>>()
        .push(PlaceTile {
            x: 5,
            y: 5,
            building_type: Some(BuildingType::Miner),
        });

    app.update();
    app.update();

    let grid = app.world().resource::<Grid>();
    let entity = *grid.occupancy.get(&(5, 5)).expect("(5,5) must be occupied");

    let pos = app
        .world()
        .get::<Position>(entity)
        .expect("Position component must be present");
    assert_eq!(pos, &Position { x: 5, y: 5 });

    let building = app
        .world()
        .get::<Building>(entity)
        .expect("Building component must be present for typed placement");
    assert_eq!(building.building_type, BuildingType::Miner);
}
