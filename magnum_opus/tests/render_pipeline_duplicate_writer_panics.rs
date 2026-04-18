//! F18 render-pipeline / AC3: two modules claiming writes: names![RenderPipelineConfig] panic single-writer.

use magnum_opus::core::*;
use magnum_opus::names;
use magnum_opus::render_pipeline::{RenderPipelineConfig, RenderPipelineConfigModule};

struct RogueConfigWriter;
impl StaticData for RogueConfigWriter {
    const ID: &'static str = "rogue_config_writer";
    fn writes() -> &'static [TypeKey] {
        names![RenderPipelineConfig]
    }
    fn metrics() -> &'static [MetricDesc] {
        &[]
    }
    fn install(ctx: &mut DataInstaller) {
        ctx.insert_resource(RenderPipelineConfig {
            low_res_width: 1,
            low_res_height: 1,
            outline_enabled: false,
            toon_bands: 0,
            posterize_levels: 0,
        });
    }
}

#[test]
#[should_panic(expected = "single-writer")]
fn second_module_claiming_config_writes_panics() {
    let _ = Harness::new()
        .with_data::<RenderPipelineConfigModule>()
        .with_data::<RogueConfigWriter>()
        .build();
}
