//! Sobel outline Material2d for F22 render-outline.
//!
//! `OutlineMaterial` wraps the low-res framebuffer as a sampled texture and a
//! uniform block with edge detection parameters. The fragment shader at
//! `outline.wgsl` computes a 3x3 Sobel over luminance and returns the outline
//! color on edges or the source sample otherwise.

use bevy::asset::Asset;
use bevy::color::LinearRgba;
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;
use bevy::sprite_render::Material2d;

#[derive(ShaderType, Clone, Debug, PartialEq)]
pub struct OutlineParams {
    pub threshold: f32,
    pub color: LinearRgba,
}

impl Default for OutlineParams {
    fn default() -> Self {
        Self {
            threshold: 0.08,
            color: LinearRgba::BLACK,
        }
    }
}

#[derive(AsBindGroup, Asset, TypePath, Clone, Debug)]
pub struct OutlineMaterial {
    #[uniform(0)]
    pub params: OutlineParams,
    #[texture(1)]
    #[sampler(2)]
    pub source: Handle<Image>,
}

impl Material2d for OutlineMaterial {
    fn fragment_shader() -> ShaderRef {
        "embedded://magnum_opus/render_pipeline/outline.wgsl".into()
    }
}
