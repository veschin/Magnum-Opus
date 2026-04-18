use super::resource::RenderPipelineConfig;
use crate::core::*;
use crate::names;

pub struct RenderPipelineConfigModule;

impl StaticData for RenderPipelineConfigModule {
    const ID: &'static str = "render_pipeline_config";

    fn writes() -> &'static [TypeKey] {
        names![RenderPipelineConfig]
    }

    fn metrics() -> &'static [MetricDesc] {
        &[]
    }

    fn install(ctx: &mut DataInstaller) {
        ctx.insert_resource(RenderPipelineConfig {
            low_res_width: 480,
            low_res_height: 270,
            outline_enabled: false,
            toon_bands: 0,
            posterize_levels: 0,
        });
    }
}
