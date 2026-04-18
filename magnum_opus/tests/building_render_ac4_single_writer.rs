//! F-building-render AC4 - second View claiming BuildingSceneCache writes panics.

use magnum_opus::building_render::{BuildingRenderModule, BuildingSceneCache};
use magnum_opus::buildings::BuildingDbModule;
use magnum_opus::core::*;
use magnum_opus::grid::GridModule;
use magnum_opus::group_formation::GroupFormationModule;
use magnum_opus::landscape::LandscapeModule;
use magnum_opus::manifold::ManifoldModule;
use magnum_opus::names;
use magnum_opus::placement::PlacementInputModule;
use magnum_opus::recipes_production::{ProductionModule, RecipeDbModule};
use magnum_opus::render_pipeline::RenderPipelineConfigModule;
use magnum_opus::resources::ResourcesModule;
use magnum_opus::world_config::WorldConfigModule;
use magnum_opus::world_render::WorldRenderModule;

struct RogueBuildingRenderWriter;
impl View for RogueBuildingRenderWriter {
    const ID: &'static str = "rogue_building_render_writer";
    fn reads() -> &'static [TypeKey] {
        &[]
    }
    fn writes() -> &'static [TypeKey] {
        names![BuildingSceneCache]
    }
    fn metrics() -> &'static [MetricDesc] {
        &[]
    }
    fn install(ctx: &mut ViewInstaller) {
        ctx.write_resource::<BuildingSceneCache>();
    }
}

#[test]
#[should_panic(expected = "single-writer")]
fn second_writer_of_building_scene_cache_panics() {
    let _ = Harness::new()
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
        .with_view::<RogueBuildingRenderWriter>()
        .with_input::<PlacementInputModule>()
        .build();
}
