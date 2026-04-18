//! F6 AC2 - Miner's OutputBuffer is drained each tick after collection.

use magnum_opus::buildings::{BuildingDbModule, BuildingType};
use magnum_opus::core::{AppExt, CommandBus, Harness};
use magnum_opus::grid::{Grid, GridModule, PlaceTile};
use magnum_opus::group_formation::GroupFormationModule;
use magnum_opus::manifold::ManifoldModule;
use magnum_opus::placement::PlacementInputModule;
use magnum_opus::recipes_production::{OutputBuffer, ProductionModule, RecipeDbModule};
use magnum_opus::world_config::WorldConfigModule;

#[test]
fn output_buffer_is_drained_after_manifold_collection() {
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
            x: 4,
            y: 4,
            building_type: Some(BuildingType::Miner),
        });

    for _ in 0..12 {
        app.update();
    }

    let grid = app.world().resource::<Grid>();
    let miner = *grid.occupancy.get(&(4, 4)).unwrap();
    let output = app.world().get::<OutputBuffer>(miner).unwrap();
    let sum: f32 = output.slots.values().sum();
    assert_eq!(
        sum, 0.0,
        "Miner's OutputBuffer must be empty after manifold collect"
    );
}
