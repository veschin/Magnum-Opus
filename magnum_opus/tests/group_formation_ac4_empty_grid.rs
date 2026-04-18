//! F7 AC4 - empty grid yields no groups regardless of how many ticks run.

use magnum_opus::buildings::BuildingDbModule;
use magnum_opus::core::{AppExt, Harness};
use magnum_opus::grid::GridModule;
use magnum_opus::group_formation::{GroupFormationModule, GroupIndex};
use magnum_opus::placement::PlacementInputModule;
use magnum_opus::world_config::WorldConfigModule;

#[test]
fn empty_grid_produces_no_groups() {
    let mut app = Harness::new()
        .with_data::<WorldConfigModule>()
        .with_data::<BuildingDbModule>()
        .with_sim::<GridModule>()
        .with_sim::<GroupFormationModule>()
        .with_input::<PlacementInputModule>()
        .build();

    for _ in 0..5 {
        app.update();
    }

    let index = app.world().resource::<GroupIndex>();
    assert!(index.groups.is_empty());
    assert!(index.member_to_group.is_empty());
}
