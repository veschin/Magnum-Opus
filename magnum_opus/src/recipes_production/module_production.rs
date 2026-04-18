//! Production SimDomain running in Phase::Production.

use super::resource::RecipeDB;
use super::systems::{production_advance_system, production_attach_system};
use crate::buildings::BuildingDB;
use crate::core::*;
use crate::names;

pub struct ProductionModule;

impl SimDomain for ProductionModule {
    const ID: &'static str = "production";
    const PRIMARY_PHASE: Phase = Phase::Production;

    fn contract() -> SimContract {
        SimContract {
            reads: names![RecipeDB, BuildingDB],
            ..SimContract::EMPTY
        }
    }

    fn install(ctx: &mut SimInstaller) {
        ctx.read_resource::<RecipeDB>();
        ctx.read_resource::<BuildingDB>();
        ctx.add_system(production_attach_system);
        ctx.add_system(production_advance_system);
    }
}
