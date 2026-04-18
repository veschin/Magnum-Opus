//! F5a AC3 - a Miner accumulates IronOre in OutputBuffer after several ticks.

use magnum_opus::buildings::{BuildingDbModule, BuildingType};
use magnum_opus::core::{AppExt, CommandBus, Harness};
use magnum_opus::grid::{Grid, GridModule, PlaceTile};
use magnum_opus::group_formation::GroupFormationModule;
use magnum_opus::placement::PlacementInputModule;
use magnum_opus::recipes_production::{
    OutputBuffer, ProductionModule, RecipeDbModule, ResourceType,
};
use magnum_opus::world_config::WorldConfigModule;

#[test]
fn miner_accumulates_iron_ore() {
    let mut app = Harness::new()
        .with_data::<WorldConfigModule>()
        .with_data::<BuildingDbModule>()
        .with_data::<RecipeDbModule>()
        .with_sim::<GridModule>()
        .with_sim::<GroupFormationModule>()
        .with_sim::<ProductionModule>()
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
    let entity = *grid.occupancy.get(&(10, 10)).unwrap();
    let buf = app.world().get::<OutputBuffer>(entity).unwrap();
    let ore = buf.slots.get(&ResourceType::IronOre).copied().unwrap_or(0.0);
    assert!(
        ore >= 1.0,
        "Miner should have produced at least one IronOre (got {ore})"
    );
}
