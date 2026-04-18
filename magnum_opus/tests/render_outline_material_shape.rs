//! F22 AC1, AC2 - PostProcessMaterial + PostProcessParams shape checks.
//!
//! Verifies that the public surface of the post-process material exposes the
//! Sobel outline parameters and posterize level expected by the pipeline.

use bevy::color::LinearRgba;
use bevy::prelude::*;

use magnum_opus::render_pipeline::{PostProcessMaterial, PostProcessParams};

#[test]
fn post_process_params_defaults_enable_outline_and_posterize() {
    let params = PostProcessParams::default();

    assert_eq!(params.outline_threshold, 0.18);
    assert_eq!(params.outline_color, LinearRgba::BLACK);
    assert_eq!(params.posterize_levels, 8.0);
    assert_eq!(params.outline_enabled, 1.0);
}

#[test]
fn post_process_material_constructs_with_defaults() {
    let material = PostProcessMaterial {
        params: PostProcessParams::default(),
        source: Handle::<Image>::default(),
    };

    assert_eq!(material.params.outline_threshold, 0.18);
    assert_eq!(material.params.outline_color, LinearRgba::BLACK);
    assert_eq!(material.params.posterize_levels, 8.0);
}
