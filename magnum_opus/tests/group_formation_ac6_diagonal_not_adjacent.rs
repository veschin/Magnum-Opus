//! F7 AC6 - diagonal neighbors are NOT considered adjacent.

use magnum_opus::buildings::{BuildingDbModule, BuildingType};
use magnum_opus::core::{AppExt, CommandBus, Harness};
use magnum_opus::grid::{GridModule, PlaceTile};
use magnum_opus::group_formation::{GroupFormationModule, GroupIndex};
use magnum_opus::placement::PlacementInputModule;
use magnum_opus::world_config::WorldConfigModule;

#[test]
fn diagonal_neighbors_yield_two_groups() {
    let mut app = Harness::new()
        .with_data::<WorldConfigModule>()
        .with_data::<BuildingDbModule>()
        .with_sim::<GridModule>()
        .with_sim::<GroupFormationModule>()
        .with_input::<PlacementInputModule>()
        .build();

    {
        let mut bus = app.world_mut().resource_mut::<CommandBus<PlaceTile>>();
        bus.push(PlaceTile {
            x: 5,
            y: 5,
            building_type: Some(BuildingType::EnergySource),
        });
        bus.push(PlaceTile {
            x: 6,
            y: 6,
            building_type: Some(BuildingType::EnergySource),
        });
    }

    app.update();
    app.update();
    app.update();

    let index = app.world().resource::<GroupIndex>();
    assert_eq!(index.groups.len(), 2);
    assert_eq!(index.member_to_group.len(), 2);
}
