//! F7 AC1 - three cardinal-adjacent Buildings merge into one group.

use magnum_opus::buildings::{BuildingDbModule, BuildingType};
use magnum_opus::core::{AppExt, CommandBus, Harness};
use magnum_opus::grid::{GridModule, PlaceTile};
use magnum_opus::group_formation::{GroupFormationModule, GroupIndex, GroupMember};
use magnum_opus::placement::PlacementInputModule;
use magnum_opus::world_config::WorldConfigModule;

#[test]
fn three_adjacent_buildings_form_one_group() {
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
            building_type: Some(BuildingType::Miner),
        });
        bus.push(PlaceTile {
            x: 5,
            y: 6,
            building_type: Some(BuildingType::Miner),
        });
        bus.push(PlaceTile {
            x: 6,
            y: 5,
            building_type: Some(BuildingType::Miner),
        });
    }

    app.update();
    app.update();
    app.update();

    let index = app.world().resource::<GroupIndex>();
    assert_eq!(
        index.groups.len(),
        1,
        "three cardinal-adjacent Buildings must share one group"
    );
    assert_eq!(index.member_to_group.len(), 3);

    let mut group_ids = std::collections::BTreeSet::new();
    for (_member, group) in &index.member_to_group {
        group_ids.insert(*group);
    }
    assert_eq!(group_ids.len(), 1, "all members must reference the same group");

    let grid = app.world().resource::<magnum_opus::grid::Grid>();
    for tile in [(5, 5), (5, 6), (6, 5)] {
        let entity = *grid.occupancy.get(&tile).unwrap();
        let gm = app
            .world()
            .get::<GroupMember>(entity)
            .expect("every Building in the cluster must carry GroupMember");
        assert!(index.groups.contains(&gm.group));
    }
}
