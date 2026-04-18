//! F7 AC2 - two disjoint Building clusters yield two groups.

use magnum_opus::buildings::{BuildingDbModule, BuildingType};
use magnum_opus::core::{AppExt, CommandBus, Harness};
use magnum_opus::grid::{Grid, GridModule, PlaceTile};
use magnum_opus::group_formation::{GroupFormationModule, GroupIndex, GroupMember};
use magnum_opus::placement::PlacementInputModule;
use magnum_opus::world_config::WorldConfigModule;

#[test]
fn disjoint_clusters_yield_separate_groups() {
    let mut app = Harness::new()
        .with_data::<WorldConfigModule>()
        .with_data::<BuildingDbModule>()
        .with_sim::<GridModule>()
        .with_sim::<GroupFormationModule>()
        .with_input::<PlacementInputModule>()
        .build();

    {
        let mut bus = app.world_mut().resource_mut::<CommandBus<PlaceTile>>();
        for (x, y) in [(3, 3), (3, 4), (20, 20), (20, 21)] {
            bus.push(PlaceTile {
                x,
                y,
                building_type: Some(BuildingType::Smelter),
            });
        }
    }

    app.update();
    app.update();
    app.update();

    let index = app.world().resource::<GroupIndex>();
    assert_eq!(index.groups.len(), 2);
    assert_eq!(index.member_to_group.len(), 4);

    let grid = app.world().resource::<Grid>();
    let group_of = |x: u32, y: u32| {
        let e = *grid.occupancy.get(&(x, y)).unwrap();
        app.world().get::<GroupMember>(e).unwrap().group
    };

    let g_a = group_of(3, 3);
    let g_b = group_of(3, 4);
    let g_c = group_of(20, 20);
    let g_d = group_of(20, 21);

    assert_eq!(g_a, g_b, "same cluster must share group");
    assert_eq!(g_c, g_d, "same cluster must share group");
    assert_ne!(g_a, g_c, "disjoint clusters must have distinct groups");
}
