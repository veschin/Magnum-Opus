//! RecipeDB StaticData with MVP recipe table.

use super::resource::RecipeDB;
use super::types::{RecipeDef, ResourceType};
use crate::buildings::BuildingType;
use crate::core::*;
use crate::names;
use std::collections::BTreeMap;

pub struct RecipeDbModule;

impl StaticData for RecipeDbModule {
    const ID: &'static str = "recipe_db";

    fn writes() -> &'static [TypeKey] {
        names![RecipeDB]
    }

    fn metrics() -> &'static [MetricDesc] {
        &[]
    }

    fn install(ctx: &mut DataInstaller) {
        let mut recipes = BTreeMap::new();
        recipes.insert(
            BuildingType::Miner,
            RecipeDef {
                inputs: vec![],
                outputs: vec![(ResourceType::IronOre, 1.0)],
                duration_ticks: 4,
            },
        );
        recipes.insert(
            BuildingType::Smelter,
            RecipeDef {
                inputs: vec![(ResourceType::IronOre, 2.0)],
                outputs: vec![(ResourceType::IronBar, 1.0)],
                duration_ticks: 4,
            },
        );
        recipes.insert(
            BuildingType::Mall,
            RecipeDef {
                inputs: vec![],
                outputs: vec![],
                duration_ticks: 1,
            },
        );
        recipes.insert(
            BuildingType::EnergySource,
            RecipeDef {
                inputs: vec![],
                outputs: vec![],
                duration_ticks: 1,
            },
        );
        ctx.insert_resource(RecipeDB { recipes });
    }
}
