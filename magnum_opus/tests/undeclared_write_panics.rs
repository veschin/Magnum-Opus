use bevy::prelude::*;
use magnum_opus::core::*;
use magnum_opus::names;

#[derive(Resource, Default)]
struct Declared;

#[derive(Resource, Default)]
struct NotDeclared;

struct Sneaky;
impl SimDomain for Sneaky {
    const ID: &'static str = "sneaky";
    const PRIMARY_PHASE: Phase = Phase::World;
    fn contract() -> SimContract {
        SimContract {
            writes: names![Declared],
            ..SimContract::EMPTY
        }
    }
    fn install(ctx: &mut SimInstaller) {
        ctx.write_resource::<Declared>();
        ctx.write_resource::<NotDeclared>();
    }
}

#[test]
#[should_panic(expected = "not in contract.writes")]
fn undeclared_write_panics() {
    let _ = Harness::new().with_sim::<Sneaky>().build();
}
