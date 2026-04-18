//! F6 AC3 - Two adjacent Miners share one Manifold pool, contributions add up.

use magnum_opus::buildings::{BuildingDbModule, BuildingType};
use magnum_opus::core::{AppExt, CommandBus, Harness};
use magnum_opus::grid::{Grid, GridModule, PlaceTile};
use magnum_opus::group_formation::{GroupFormationModule, GroupMember};
use magnum_opus::manifold::{Manifold, ManifoldModule};
use magnum_opus::placement::PlacementInputModule;
use magnum_opus::recipes_production::{ProductionModule, RecipeDbModule, ResourceType};
use magnum_opus::world_config::WorldConfigModule;

#[test]
fn two_adjacent_miners_share_pool() {
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
            x: 5,
            y: 5,
            building_type: Some(BuildingType::Miner),
        });
        bus.push(PlaceTile {
            x: 5,
            y: 6,
            building_type: Some(BuildingType::Miner),
        });
    }

    for _ in 0..16 {
        app.update();
    }

    let grid = app.world().resource::<Grid>();
    let m1 = *grid.occupancy.get(&(5, 5)).unwrap();
    let m2 = *grid.occupancy.get(&(5, 6)).unwrap();
    let g1 = app.world().get::<GroupMember>(m1).unwrap().group;
    let g2 = app.world().get::<GroupMember>(m2).unwrap().group;
    assert_eq!(g1, g2, "the two Miners must share a group");

    let manifold = app.world().get::<Manifold>(g1).unwrap();
    let ore = manifold
        .slots
        .get(&ResourceType::IronOre)
        .copied()
        .unwrap_or(0.0);
    assert!(
        ore >= 2.0,
        "shared pool must accumulate from both miners (got {ore})"
    );
}
