//! F18 render-pipeline / AC2: RenderPipelineConfig is plain data - Resource, Clone, Debug, PartialEq.
//!
//! Compile-time assertion plus a small runtime equality check. Any impl that drops
//! one of these derives breaks this file.

use magnum_opus::render_pipeline::RenderPipelineConfig;

fn _assert_resource<T: bevy::prelude::Resource>() {}
fn _assert_clone<T: Clone>() {}
fn _assert_debug<T: std::fmt::Debug>() {}
fn _assert_partial_eq<T: PartialEq>() {}

#[test]
fn config_has_required_derives() {
    _assert_resource::<RenderPipelineConfig>();
    _assert_clone::<RenderPipelineConfig>();
    _assert_debug::<RenderPipelineConfig>();
    _assert_partial_eq::<RenderPipelineConfig>();

    let a = RenderPipelineConfig {
        low_res_width: 480,
        low_res_height: 270,
        outline_enabled: false,
        toon_bands: 0,
        posterize_levels: 0,
    };
    let b = a.clone();
    assert_eq!(a, b);
    let _fmt = format!("{a:?}");
}
