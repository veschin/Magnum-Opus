//! F6 AC1 - Miner's production ends up in the group's Manifold pool.

use magnum_opus::buildings::{BuildingDbModule, BuildingType};
use magnum_opus::core::{AppExt, CommandBus, Harness};
use magnum_opus::grid::{Grid, GridModule, PlaceTile};
use magnum_opus::group_formation::{GroupFormationModule, GroupMember};
use magnum_opus::manifold::{Manifold, ManifoldModule};
use magnum_opus::placement::PlacementInputModule;
use magnum_opus::recipes_production::{ProductionModule, RecipeDbModule, ResourceType};
use magnum_opus::world_config::WorldConfigModule;

#[test]
fn miner_output_reaches_group_manifold() {
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

    app.world_mut()
        .resource_mut::<CommandBus<PlaceTile>>()
        .push(PlaceTile {
            x: 10,
            y: 10,
            building_type: Some(BuildingType::Miner),
        });

    for _ in 0..12 {
        app.update();
    }

    let grid = app.world().resource::<Grid>();
    let miner = *grid.occupancy.get(&(10, 10)).unwrap();
    let gm = app.world().get::<GroupMember>(miner).unwrap();
    let manifold = app.world().get::<Manifold>(gm.group).expect("Group must carry Manifold");
    let ore = manifold
        .slots
        .get(&ResourceType::IronOre)
        .copied()
        .unwrap_or(0.0);
    assert!(
        ore >= 1.0,
        "group Manifold must accumulate IronOre from Miner (got {ore})"
    );
}
