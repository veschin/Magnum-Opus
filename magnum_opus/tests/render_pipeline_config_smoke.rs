//! F18 render-pipeline / AC1: RenderPipelineConfig holds MVP constants after build.

use magnum_opus::core::*;
use magnum_opus::render_pipeline::{RenderPipelineConfig, RenderPipelineConfigModule};

#[test]
fn render_pipeline_config_inserted_with_mvp_values() {
    let app = Harness::new()
        .with_data::<RenderPipelineConfigModule>()
        .build();
    let cfg = app.world().resource::<RenderPipelineConfig>();
    assert_eq!(cfg.low_res_width, 480);
    assert_eq!(cfg.low_res_height, 270);
    assert!(!cfg.outline_enabled);
    assert_eq!(cfg.toon_bands, 0);
    assert_eq!(cfg.posterize_levels, 0);
}
