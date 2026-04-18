//! F5a AC4 - Smelter without inputs never produces IronBar.

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
fn smelter_without_inputs_does_not_produce() {
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
            x: 2,
            y: 2,
            building_type: Some(BuildingType::Smelter),
        });

    for _ in 0..20 {
        app.update();
    }

    let grid = app.world().resource::<Grid>();
    let entity = *grid.occupancy.get(&(2, 2)).unwrap();
    let buf = app.world().get::<OutputBuffer>(entity).unwrap();
    let bar = buf.slots.get(&ResourceType::IronBar).copied().unwrap_or(0.0);
    assert_eq!(bar, 0.0, "Smelter with empty inputs must not produce bars");
}
