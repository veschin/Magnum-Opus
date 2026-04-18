//! F-building-render AC2 - four building types yield four sprite entities.

use magnum_opus::building_render::{BuildingRenderModule, BuildingSceneCache};
use magnum_opus::buildings::{BuildingDbModule, BuildingType};
use magnum_opus::core::{AppExt, CommandBus, Harness};
use magnum_opus::grid::{GridModule, PlaceTile};
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
fn four_building_types_yield_four_sprites() {
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

    {
        let mut bus = app.world_mut().resource_mut::<CommandBus<PlaceTile>>();
        for (x, y, t) in [
            (3, 3, BuildingType::Miner),
            (10, 10, BuildingType::Smelter),
            (20, 20, BuildingType::Mall),
            (30, 30, BuildingType::EnergySource),
        ] {
            bus.push(PlaceTile {
                x,
                y,
                building_type: Some(t),
            });
        }
    }

    for _ in 0..4 {
        app.update();
    }

    let cache = app.world().resource::<BuildingSceneCache>();
    assert_eq!(cache.entities.len(), 4);
}
