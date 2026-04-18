//! Manifold component attached to every Group entity.

use crate::recipes_production::ResourceType;
use bevy::prelude::Component;
use std::collections::BTreeMap;

#[derive(Component, Debug, Default, Clone)]
pub struct Manifold {
    pub slots: BTreeMap<ResourceType, f32>,
}
