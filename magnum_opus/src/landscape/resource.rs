use bevy::prelude::Resource;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub enum TerrainKind {
    #[default]
    Grass,
    Rock,
    Water,
    Lava,
    Sand,
    Mountain,
    Pit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TerrainCell {
    pub kind: TerrainKind,
    pub elevation: i8,
    pub depth: u8,
    pub moisture: u8,
}

#[derive(Resource, Default, Debug)]
pub struct Landscape {
    pub width: u32,
    pub height: u32,
    pub cells: Vec<TerrainCell>,
    pub ready: bool,
}
