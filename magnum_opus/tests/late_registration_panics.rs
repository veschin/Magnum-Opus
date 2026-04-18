use bevy::prelude::*;
use magnum_opus::core::*;
use magnum_opus::names;

#[derive(Resource, Default)]
struct ResY;

struct Valid;
impl SimDomain for Valid {
    const ID: &'static str = "valid";
    const PRIMARY_PHASE: Phase = Phase::World;
    fn contract() -> SimContract {
        SimContract {
            writes: names![ResY],
            ..SimContract::EMPTY
        }
    }
    fn install(ctx: &mut SimInstaller) {
        ctx.write_resource::<ResY>();
        ctx.add_system(|| {});
    }
}

#[derive(Resource, Default)]
struct ResZ;

struct Late;
impl SimDomain for Late {
    const ID: &'static str = "late";
    const PRIMARY_PHASE: Phase = Phase::World;
    fn contract() -> SimContract {
        SimContract {
            writes: names![ResZ],
            ..SimContract::EMPTY
        }
    }
    fn install(ctx: &mut SimInstaller) {
        ctx.write_resource::<ResZ>();
        ctx.add_system(|| {});
    }
}

#[test]
#[should_panic(expected = "registry is frozen")]
fn registering_after_finalize_panics() {
    let mut app = Harness::new().with_sim::<Valid>().build();
    app.add_sim::<Late>();
}
