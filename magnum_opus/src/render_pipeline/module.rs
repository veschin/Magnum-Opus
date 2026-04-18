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
        ctx.insert_resource(RenderPipelineConfig::default());
    }
}
