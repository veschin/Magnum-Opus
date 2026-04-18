use bevy::prelude::Resource;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceKind {
    IronOre,
    CopperOre,
    Stone,
    Coal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quality {
    Normal,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vein {
    pub kind: ResourceKind,
    pub quality: Quality,
    pub remaining: f32,
}

#[derive(Resource, Default, Debug)]
pub struct ResourceVeins {
    pub veins: BTreeMap<(u32, u32), Vein>,
    pub clusters: u32,
    pub ready: bool,
}
