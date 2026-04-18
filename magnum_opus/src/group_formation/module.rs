//! Group-formation SimDomain running in Phase::Groups.

use super::resource::GroupIndex;
use super::systems::group_formation_system;
use crate::core::*;
use crate::names;

pub struct GroupFormationModule;

impl SimDomain for GroupFormationModule {
    const ID: &'static str = "group_formation";
    const PRIMARY_PHASE: Phase = Phase::Groups;

    fn contract() -> SimContract {
        SimContract {
            writes: names![GroupIndex],
            ..SimContract::EMPTY
        }
    }

    fn install(ctx: &mut SimInstaller) {
        ctx.write_resource::<GroupIndex>();
        ctx.add_system(group_formation_system);
    }
}
