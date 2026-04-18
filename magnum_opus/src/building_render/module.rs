//! View module that owns BuildingSceneCache and runs the sprite diff-sync.

use super::resource::BuildingSceneCache;
use super::systems::building_render_system;
use crate::core::*;
use crate::landscape::Landscape;
use crate::names;

pub struct BuildingRenderModule;

impl View for BuildingRenderModule {
    const ID: &'static str = "building_render";

    fn reads() -> &'static [TypeKey] {
        names![Landscape]
    }

    fn writes() -> &'static [TypeKey] {
        names![BuildingSceneCache]
    }

    fn metrics() -> &'static [MetricDesc] {
        &[]
    }

    fn install(ctx: &mut ViewInstaller) {
        ctx.write_resource::<BuildingSceneCache>();
        ctx.read_resource::<Landscape>();
        ctx.add_system(building_render_system);
    }
}
