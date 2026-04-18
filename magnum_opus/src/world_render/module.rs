use super::resource::WorldSceneCache;
use super::systems::world_render_system;
use crate::core::*;
use crate::landscape::Landscape;
use crate::names;
use crate::resources::ResourceVeins;

pub struct WorldRenderModule;

impl View for WorldRenderModule {
    const ID: &'static str = "world_render";

    fn reads() -> &'static [TypeKey] {
        names![Landscape, ResourceVeins]
    }

    fn writes() -> &'static [TypeKey] {
        names![WorldSceneCache]
    }

    fn metrics() -> &'static [MetricDesc] {
        &[]
    }

    fn install(ctx: &mut ViewInstaller) {
        ctx.write_resource::<WorldSceneCache>();
        ctx.read_resource::<Landscape>();
        ctx.read_resource::<ResourceVeins>();
        ctx.add_system(world_render_system);
    }
}
