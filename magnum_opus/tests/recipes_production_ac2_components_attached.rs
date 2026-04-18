//! F5a AC2 - production components attached to placed Building entities.

use magnum_opus::buildings::{BuildingDbModule, BuildingType};
use magnum_opus::core::{AppExt, CommandBus, Harness};
use magnum_opus::grid::{Grid, GridModule, PlaceTile};
use magnum_opus::group_formation::GroupFormationModule;
use magnum_opus::placement::PlacementInputModule;
use magnum_opus::recipes_production::{
    InputBuffer, OutputBuffer, ProductionModule, ProductionState, Recipe, RecipeDbModule,
};
use magnum_opus::world_config::WorldConfigModule;

#[test]
fn production_components_attached_after_ticks() {
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
            x: 5,
            y: 5,
            building_type: Some(BuildingType::Miner),
        });

    for _ in 0..5 {
        app.update();
    }

    let grid = app.world().resource::<Grid>();
    let entity = *grid.occupancy.get(&(5, 5)).unwrap();
    assert!(app.world().get::<Recipe>(entity).is_some());
    assert!(app.world().get::<ProductionState>(entity).is_some());
    assert!(app.world().get::<OutputBuffer>(entity).is_some());
    assert!(app.world().get::<InputBuffer>(entity).is_some());
}
