//! Fullscreen post-process Material2d that runs the pixel-art shader chain on
//! the low-res framebuffer produced by the scene camera.
//!
//! Pipeline order inside the fragment shader: Sobel-over-luminance edge
//! detection produces the outline mask; source pixels pass through a
//! per-channel posterize step; outline pixels override the posterized colour.
//! Edge detection uses `textureDimensions(source)` so its texel offsets are
//! sized to the low-res source (e.g. 480×270), not the upscaled window. This
//! keeps outlines a single low-res pixel thick after the nearest-neighbour
//! blit.

use bevy::asset::Asset;
use bevy::color::LinearRgba;
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;
use bevy::sprite_render::Material2d;

#[derive(ShaderType, Clone, Debug, PartialEq)]
pub struct PostProcessParams {
    pub outline_color: LinearRgba,
    pub outline_threshold: f32,
    pub posterize_levels: f32,
    pub outline_enabled: f32,
    pub _pad: f32,
}

impl Default for PostProcessParams {
    fn default() -> Self {
        Self {
            outline_color: LinearRgba::BLACK,
            outline_threshold: 0.18,
            posterize_levels: 8.0,
            outline_enabled: 1.0,
            _pad: 0.0,
        }
    }
}

#[derive(AsBindGroup, Asset, TypePath, Clone, Debug)]
pub struct PostProcessMaterial {
    #[uniform(0)]
    pub params: PostProcessParams,
    #[texture(1)]
    #[sampler(2)]
    pub source: Handle<Image>,
}

impl Material2d for PostProcessMaterial {
    fn fragment_shader() -> ShaderRef {
        "embedded://magnum_opus/render_pipeline/post_process.wgsl".into()
    }
}
