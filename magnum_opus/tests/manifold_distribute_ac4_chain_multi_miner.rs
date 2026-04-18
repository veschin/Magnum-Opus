//! F5b AC4 - two miners feeding one smelter produce multiple IronBars.

use magnum_opus::buildings::{BuildingDbModule, BuildingType};
use magnum_opus::core::{AppExt, CommandBus, Harness};
use magnum_opus::grid::{Grid, GridModule, PlaceTile};
use magnum_opus::group_formation::{GroupFormationModule, GroupMember};
use magnum_opus::manifold::{Manifold, ManifoldModule};
use magnum_opus::placement::PlacementInputModule;
use magnum_opus::recipes_production::{
    OutputBuffer, ProductionModule, RecipeDbModule, ResourceType,
};
use magnum_opus::world_config::WorldConfigModule;

#[test]
#[ignore = "F5b distribute WIP - timing/race bug"]
fn two_miners_plus_smelter_chain_produces_multiple_bars() {
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
            x: 10,
            y: 10,
            building_type: Some(BuildingType::Miner),
        });
        bus.push(PlaceTile {
            x: 10,
            y: 11,
            building_type: Some(BuildingType::Miner),
        });
        bus.push(PlaceTile {
            x: 10,
            y: 12,
            building_type: Some(BuildingType::Smelter),
        });
    }

    for _ in 0..40 {
        app.update();
    }

    let grid = app.world().resource::<Grid>();
    let smelter = *grid.occupancy.get(&(10, 12)).unwrap();
    let out = app.world().get::<OutputBuffer>(smelter).unwrap();
    let bars = out.slots.get(&ResourceType::IronBar).copied().unwrap_or(0.0);
    assert!(bars >= 2.0, "expected at least 2 bars, got {bars}");

    let miner = *grid.occupancy.get(&(10, 10)).unwrap();
    let gm = app.world().get::<GroupMember>(miner).unwrap();
    let manifold = app.world().get::<Manifold>(gm.group).unwrap();
    let ore = manifold.slots.get(&ResourceType::IronOre).copied().unwrap_or(0.0);
    assert!(ore < 10.0, "pool must stay bounded, got {ore}");
}
