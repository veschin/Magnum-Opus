use bevy::prelude::Resource;

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct RenderPipelineConfig {
    pub low_res_width: u32,
    pub low_res_height: u32,
    pub outline_enabled: bool,
    pub toon_bands: u8,
    pub posterize_levels: u8,
}
