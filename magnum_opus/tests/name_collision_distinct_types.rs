use magnum_opus::core::*;
use magnum_opus::names;

mod a {
    #[derive(bevy::prelude::Resource, Default)]
    pub struct Grid;
}

mod b {
    #[derive(bevy::prelude::Resource, Default)]
    pub struct Grid;
}

struct ModA;
impl SimDomain for ModA {
    const ID: &'static str = "mod_a";
    const PRIMARY_PHASE: Phase = Phase::World;
    fn contract() -> SimContract {
        SimContract {
            writes: names![a::Grid],
            ..SimContract::EMPTY
        }
    }
    fn install(ctx: &mut SimInstaller) {
        ctx.write_resource::<a::Grid>();
        ctx.add_system(|| {});
    }
}

struct ModB;
impl SimDomain for ModB {
    const ID: &'static str = "mod_b";
    const PRIMARY_PHASE: Phase = Phase::World;
    fn contract() -> SimContract {
        SimContract {
            writes: names![b::Grid],
            ..SimContract::EMPTY
        }
    }
    fn install(ctx: &mut SimInstaller) {
        ctx.write_resource::<b::Grid>();
        ctx.add_system(|| {});
    }
}

#[test]
fn different_types_same_simple_name_coexist() {
    let app = Harness::new().with_sim::<ModA>().with_sim::<ModB>().build();
    let reg = app.world().resource::<ModuleRegistry>();
    assert_eq!(reg.len(), 3);
    assert_eq!(reg.writer_of_type::<a::Grid>(), Some("mod_a"));
    assert_eq!(reg.writer_of_type::<b::Grid>(), Some("mod_b"));
}
