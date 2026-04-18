//! F7 AC5 - a second SimDomain claiming writes: names![GroupIndex] panics.

use magnum_opus::buildings::BuildingDbModule;
use magnum_opus::core::*;
use magnum_opus::grid::GridModule;
use magnum_opus::group_formation::{GroupFormationModule, GroupIndex};
use magnum_opus::names;
use magnum_opus::placement::PlacementInputModule;
use magnum_opus::world_config::WorldConfigModule;

struct RogueGroupIndexWriter;
impl SimDomain for RogueGroupIndexWriter {
    const ID: &'static str = "rogue_group_index_writer";
    const PRIMARY_PHASE: Phase = Phase::Groups;
    fn contract() -> SimContract {
        SimContract {
            writes: names![GroupIndex],
            ..SimContract::EMPTY
        }
    }
    fn install(ctx: &mut SimInstaller) {
        ctx.write_resource::<GroupIndex>();
        ctx.add_system(|| {});
    }
}

#[test]
#[should_panic(expected = "single-writer")]
fn second_writer_of_group_index_panics() {
    let _ = Harness::new()
        .with_data::<WorldConfigModule>()
        .with_data::<BuildingDbModule>()
        .with_sim::<GridModule>()
        .with_sim::<GroupFormationModule>()
        .with_sim::<RogueGroupIndexWriter>()
        .with_input::<PlacementInputModule>()
        .build();
}
