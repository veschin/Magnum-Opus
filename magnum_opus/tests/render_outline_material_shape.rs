//! F22 AC1, AC2 - OutlineMaterial and OutlineParams compile-time shape check.
//!
//! Verifies that the public surface of the outline material matches the PRD:
//! OutlineMaterial wraps a Handle<Image> and an OutlineParams uniform, and
//! OutlineParams defaults to threshold=0.08 and a black LinearRgba color.

use bevy::color::LinearRgba;
use bevy::prelude::*;

use magnum_opus::render_pipeline::{OutlineMaterial, OutlineParams};

#[test]
fn outline_params_defaults_match_prd() {
    let params = OutlineParams::default();

    assert_eq!(params.threshold, 0.08);
    assert_eq!(params.color, LinearRgba::BLACK);
}

#[test]
fn outline_material_constructs_with_defaults() {
    let material = OutlineMaterial {
        params: OutlineParams::default(),
        source: Handle::<Image>::default(),
    };

    assert_eq!(material.params.threshold, 0.08);
    assert_eq!(material.params.color, LinearRgba::BLACK);
}
