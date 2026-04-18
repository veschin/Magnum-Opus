//! Manifold SimDomain running in Phase::Manifold.

use super::systems::{manifold_collect_system, manifold_distribute_system};
use crate::core::*;
use crate::group_formation::GroupIndex;
use crate::names;
use crate::recipes_production::RecipeDB;
use bevy::prelude::IntoScheduleConfigs;

pub struct ManifoldModule;

impl SimDomain for ManifoldModule {
    const ID: &'static str = "manifold";
    const PRIMARY_PHASE: Phase = Phase::Manifold;

    fn contract() -> SimContract {
        SimContract {
            reads: names![GroupIndex, RecipeDB],
            ..SimContract::EMPTY
        }
    }

    fn install(ctx: &mut SimInstaller) {
        ctx.read_resource::<GroupIndex>();
        ctx.read_resource::<RecipeDB>();
        ctx.add_system((manifold_collect_system, manifold_distribute_system).chain());
    }
}
