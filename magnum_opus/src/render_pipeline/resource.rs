use bevy::color::LinearRgba;
use bevy::prelude::Resource;

#[derive(Resource, Debug, Clone, PartialEq)]
pub struct RenderPipelineConfig {
    pub low_res_width: u32,
    pub low_res_height: u32,

    pub outline_enabled: bool,
    pub outline_threshold: f32,
    pub outline_color: LinearRgba,

    pub toon_bands: u8,
    pub posterize_levels: u8,
}

impl Default for RenderPipelineConfig {
    fn default() -> Self {
        Self {
            low_res_width: 480,
            low_res_height: 270,

            outline_enabled: true,
            outline_threshold: 0.18,
            outline_color: LinearRgba::rgb(0.02, 0.02, 0.03),

            toon_bands: 5,
            posterize_levels: 10,
        }
    }
}
