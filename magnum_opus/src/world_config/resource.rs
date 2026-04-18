use bevy::prelude::Resource;

#[derive(Resource, Debug, Clone)]
pub struct WorldConfig {
    pub width: u32,
    pub height: u32,
    pub seed: u64,
}
