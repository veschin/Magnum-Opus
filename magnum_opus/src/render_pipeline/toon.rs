//! Flat toon Material3d for the pixel-art pipeline.
//!
//! Every scene mesh (terrain tiles, veins, buildings) uses `ToonMaterial`
//! instead of the stock PBR `StandardMaterial`. The fragment shader computes
//! `NdotL` against a single uniform sun direction, quantises the result into
//! `bands` discrete steps, and modulates `base_color` against an `ambient`
//! floor. The output has chunky flat-shaded faces with hard light/shadow
//! boundaries - the foundation for Sobel outline detection downstream.
//!
//! PBR is deliberately avoided. Specular, roughness, metallic, IBL and shadow
//! maps all smooth the signal in ways that fight the pixel-art aesthetic.

use bevy::asset::Asset;
use bevy::color::LinearRgba;
use bevy::math::Vec3;
use bevy::pbr::Material;
use bevy::reflect::TypePath;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;

#[derive(ShaderType, Clone, Debug, PartialEq)]
pub struct ToonParams {
    pub base_color: LinearRgba,
    pub ambient: LinearRgba,
    pub sun_dir: Vec3,
    pub bands: u32,
}

impl Default for ToonParams {
    fn default() -> Self {
        Self {
            base_color: LinearRgba::rgb(0.7, 0.7, 0.7),
            ambient: LinearRgba::rgb(0.20, 0.22, 0.28),
            // Asymmetric sun (strong +X, weak +Z) so the +X and +Z iso-visible
            // faces land in different bands; without this asymmetry both side
            // faces reduce to the same NdotL and the scene reads flat.
            sun_dir: Vec3::new(-1.0, -1.5, -0.3).normalize(),
            bands: 5,
        }
    }
}

#[derive(AsBindGroup, Asset, TypePath, Clone, Debug)]
pub struct ToonMaterial {
    #[uniform(0)]
    pub params: ToonParams,
}

impl Default for ToonMaterial {
    fn default() -> Self {
        Self {
            params: ToonParams::default(),
        }
    }
}

impl ToonMaterial {
    pub fn with_base(mut self, color: LinearRgba) -> Self {
        self.params.base_color = color;
        self
    }

    pub fn from_base(color: LinearRgba) -> Self {
        Self::default().with_base(color)
    }
}

impl Material for ToonMaterial {
    fn fragment_shader() -> ShaderRef {
        "embedded://magnum_opus/render_pipeline/toon.wgsl".into()
    }
}
