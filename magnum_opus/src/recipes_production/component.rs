//! Per-Building production components attached on first tick after placement.

use super::types::ResourceType;
use crate::buildings::BuildingType;
use bevy::prelude::Component;
use std::collections::BTreeMap;

#[derive(Component, Debug, Clone)]
pub struct Recipe {
    pub building_type: BuildingType,
}

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct ProductionState {
    pub progress: f32,
    pub active: bool,
}

#[derive(Component, Debug, Clone, Default)]
pub struct InputBuffer {
    pub slots: BTreeMap<ResourceType, f32>,
}

#[derive(Component, Debug, Clone, Default)]
pub struct OutputBuffer {
    pub slots: BTreeMap<ResourceType, f32>,
}
