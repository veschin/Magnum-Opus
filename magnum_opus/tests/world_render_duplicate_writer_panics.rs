//! F19 world-render / AC6: second View claiming WorldSceneCache writes panics single-writer.

use magnum_opus::core::*;
use magnum_opus::landscape::LandscapeModule;
use magnum_opus::names;
use magnum_opus::resources::ResourcesModule;
use magnum_opus::world_config::WorldConfigModule;
use magnum_opus::world_render::{WorldRenderModule, WorldSceneCache};

struct RogueCacheWriter;
impl View for RogueCacheWriter {
    const ID: &'static str = "rogue_cache_writer";
    fn reads() -> &'static [TypeKey] {
        &[]
    }
    fn writes() -> &'static [TypeKey] {
        names![WorldSceneCache]
    }
    fn metrics() -> &'static [MetricDesc] {
        &[]
    }
    fn install(ctx: &mut ViewInstaller) {
        ctx.write_resource::<WorldSceneCache>();
    }
}

#[test]
#[should_panic(expected = "single-writer")]
fn ac6_second_cache_writer_panics() {
    let _ = Harness::new()
        .with_data::<WorldConfigModule>()
        .with_sim::<LandscapeModule>()
        .with_sim::<ResourcesModule>()
        .with_view::<WorldRenderModule>()
        .with_view::<RogueCacheWriter>()
        .build();
}
