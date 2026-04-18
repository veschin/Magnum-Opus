//! F18 render-pipeline / AC1: `RenderPipelineConfig` holds the pixel-art
//! defaults after build.

use magnum_opus::core::*;
use magnum_opus::render_pipeline::{RenderPipelineConfig, RenderPipelineConfigModule};

#[test]
fn render_pipeline_config_inserted_with_pixel_art_defaults() {
    let app = Harness::new()
        .with_data::<RenderPipelineConfigModule>()
        .build();
    let cfg = app.world().resource::<RenderPipelineConfig>();
    assert_eq!(cfg.low_res_width, 480);
    assert_eq!(cfg.low_res_height, 270);
    assert!(cfg.outline_enabled);
    assert_eq!(cfg.toon_bands, 5);
    assert_eq!(cfg.posterize_levels, 8);
}
