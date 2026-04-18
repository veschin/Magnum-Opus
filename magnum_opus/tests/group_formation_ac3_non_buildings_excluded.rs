//! F7 AC3 - Position-only tiles (no Building component) are invisible to grouping.

use magnum_opus::buildings::{BuildingDbModule, BuildingType};
use magnum_opus::core::{AppExt, CommandBus, Harness};
use magnum_opus::grid::{Grid, GridModule, PlaceTile};
use magnum_opus::group_formation::{GroupFormationModule, GroupIndex, GroupMember};
use magnum_opus::placement::PlacementInputModule;
use magnum_opus::world_config::WorldConfigModule;

#[test]
fn position_only_entities_are_not_groupable_and_do_not_bridge() {
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
            x: 10,
            y: 10,
            building_type: Some(BuildingType::Mall),
        });
        bus.push(PlaceTile {
            x: 11,
            y: 10,
            building_type: None,
        });
        bus.push(PlaceTile {
            x: 12,
            y: 10,
            building_type: Some(BuildingType::Mall),
        });
    }

    app.update();
    app.update();
    app.update();

    let index = app.world().resource::<GroupIndex>();
    assert_eq!(
        index.groups.len(),
        2,
        "non-Building bridge must not connect the two clusters"
    );
    assert_eq!(index.member_to_group.len(), 2);

    let grid = app.world().resource::<Grid>();
    let bridge_entity = *grid.occupancy.get(&(11, 10)).unwrap();
    assert!(
        app.world().get::<GroupMember>(bridge_entity).is_none(),
        "Position-only entity must not carry GroupMember"
    );
}
