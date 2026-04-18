//! F6 AC4 - Disjoint groups never cross-pollinate.

use magnum_opus::buildings::{BuildingDbModule, BuildingType};
use magnum_opus::core::{AppExt, CommandBus, Harness};
use magnum_opus::grid::{Grid, GridModule, PlaceTile};
use magnum_opus::group_formation::{GroupFormationModule, GroupMember};
use magnum_opus::manifold::{Manifold, ManifoldModule};
use magnum_opus::placement::PlacementInputModule;
use magnum_opus::recipes_production::{ProductionModule, RecipeDbModule, ResourceType};
use magnum_opus::world_config::WorldConfigModule;

#[test]
fn two_disjoint_groups_do_not_share_pool() {
    let mut app = Harness::new()
        .with_data::<WorldConfigModule>()
        .with_data::<BuildingDbModule>()
        .with_data::<RecipeDbModule>()
        .with_sim::<GridModule>()
        .with_sim::<GroupFormationModule>()
        .with_sim::<ProductionModule>()
        .with_sim::<ManifoldModule>()
        .with_input::<PlacementInputModule>()
        .build();

    {
        let mut bus = app.world_mut().resource_mut::<CommandBus<PlaceTile>>();
        bus.push(PlaceTile {
            x: 3,
            y: 3,
            building_type: Some(BuildingType::Miner),
        });
        bus.push(PlaceTile {
            x: 20,
            y: 20,
            building_type: Some(BuildingType::Miner),
        });
    }

    for _ in 0..12 {
        app.update();
    }

    let grid = app.world().resource::<Grid>();
    let m1 = *grid.occupancy.get(&(3, 3)).unwrap();
    let m2 = *grid.occupancy.get(&(20, 20)).unwrap();
    let g1 = app.world().get::<GroupMember>(m1).unwrap().group;
    let g2 = app.world().get::<GroupMember>(m2).unwrap().group;
    assert_ne!(g1, g2);

    let p1 = app
        .world()
        .get::<Manifold>(g1)
        .unwrap()
        .slots
        .get(&ResourceType::IronOre)
        .copied()
        .unwrap_or(0.0);
    let p2 = app
        .world()
        .get::<Manifold>(g2)
        .unwrap()
        .slots
        .get(&ResourceType::IronOre)
        .copied()
        .unwrap_or(0.0);
    assert!(p1 >= 1.0);
    assert!(p2 >= 1.0);
    // Sum should roughly double nothing - each pool reflects its own miner only.
    assert!(p1 < 3.0);
    assert!(p2 < 3.0);
}
