//! Resource types and recipe-definition shape.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ResourceType {
    Wood,
    Stone,
    IronOre,
    IronBar,
    Coal,
}

#[derive(Debug, Clone)]
pub struct RecipeDef {
    pub inputs: Vec<(ResourceType, f32)>,
    pub outputs: Vec<(ResourceType, f32)>,
    pub duration_ticks: u32,
}
