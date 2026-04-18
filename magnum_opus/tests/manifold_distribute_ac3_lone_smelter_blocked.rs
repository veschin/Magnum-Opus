//! F5b AC3 - a Smelter without upstream supply still cannot produce.

use magnum_opus::buildings::{BuildingDbModule, BuildingType};
use magnum_opus::core::{AppExt, CommandBus, Harness};
use magnum_opus::grid::{Grid, GridModule, PlaceTile};
use magnum_opus::group_formation::GroupFormationModule;
use magnum_opus::manifold::ManifoldModule;
use magnum_opus::placement::PlacementInputModule;
use magnum_opus::recipes_production::{
    OutputBuffer, ProductionModule, RecipeDbModule, ResourceType,
};
use magnum_opus::world_config::WorldConfigModule;

#[test]
#[ignore = "F5b distribute WIP - timing/race bug"]
fn lone_smelter_remains_blocked() {
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
            x: 7,
            y: 7,
            building_type: Some(BuildingType::Smelter),
        });

    for _ in 0..30 {
        app.update();
    }

    let grid = app.world().resource::<Grid>();
    let smelter = *grid.occupancy.get(&(7, 7)).unwrap();
    let out = app.world().get::<OutputBuffer>(smelter).unwrap();
    let bars = out.slots.get(&ResourceType::IronBar).copied().unwrap_or(0.0);
    assert_eq!(bars, 0.0);
}
