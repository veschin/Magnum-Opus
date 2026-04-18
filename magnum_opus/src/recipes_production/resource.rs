//! RecipeDB resource.

use super::types::RecipeDef;
use crate::buildings::BuildingType;
use bevy::prelude::Resource;
use std::collections::BTreeMap;

#[derive(Resource, Debug, Default)]
pub struct RecipeDB {
    pub recipes: BTreeMap<BuildingType, RecipeDef>,
}
