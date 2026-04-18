//! F-building-render AC1 - placing a Miner adds its entity to BuildingSceneCache.

use magnum_opus::building_render::{BuildingRenderModule, BuildingSceneCache};
use magnum_opus::buildings::{BuildingDbModule, BuildingType};
use magnum_opus::core::{AppExt, CommandBus, Harness};
use magnum_opus::grid::{Grid, GridModule, PlaceTile};
use magnum_opus::group_formation::GroupFormationModule;
use magnum_opus::landscape::LandscapeModule;
use magnum_opus::manifold::ManifoldModule;
use magnum_opus::placement::PlacementInputModule;
use magnum_opus::recipes_production::{ProductionModule, RecipeDbModule};
use magnum_opus::render_pipeline::RenderPipelineConfigModule;
use magnum_opus::resources::ResourcesModule;
use magnum_opus::world_config::WorldConfigModule;
use magnum_opus::world_render::WorldRenderModule;

#[test]
fn placed_miner_registers_in_building_scene_cache() {
    let mut app = Harness::new()
        .with_data::<WorldConfigModule>()
        .with_data::<BuildingDbModule>()
        .with_data::<RecipeDbModule>()
        .with_data::<RenderPipelineConfigModule>()
        .with_sim::<GridModule>()
        .with_sim::<LandscapeModule>()
        .with_sim::<ResourcesModule>()
        .with_sim::<GroupFormationModule>()
        .with_sim::<ProductionModule>()
        .with_sim::<ManifoldModule>()
        .with_view::<WorldRenderModule>()
        .with_view::<BuildingRenderModule>()
        .with_input::<PlacementInputModule>()
        .build();

    app.world_mut()
        .resource_mut::<CommandBus<PlaceTile>>()
        .push(PlaceTile {
            x: 5,
            y: 5,
            building_type: Some(BuildingType::Miner),
        });

    for _ in 0..4 {
        app.update();
    }

    let grid = app.world().resource::<Grid>();
    let miner = *grid.occupancy.get(&(5, 5)).unwrap();
    let cache = app.world().resource::<BuildingSceneCache>();
    assert_eq!(cache.entities.len(), 1);
    assert!(cache.entities.contains_key(&miner));
}
