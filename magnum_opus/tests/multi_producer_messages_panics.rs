use bevy::prelude::*;
use magnum_opus::core::*;
use magnum_opus::names;

#[derive(Message)]
struct Shared;

struct A;
impl SimDomain for A {
    const ID: &'static str = "a";
    const PRIMARY_PHASE: Phase = Phase::World;
    fn contract() -> SimContract {
        SimContract {
            messages_out: names![Shared],
            ..SimContract::EMPTY
        }
    }
    fn install(ctx: &mut SimInstaller) {
        ctx.emit_message::<Shared>();
        ctx.add_system(|| {});
    }
}

struct B;
impl SimDomain for B {
    const ID: &'static str = "b";
    const PRIMARY_PHASE: Phase = Phase::Production;
    fn contract() -> SimContract {
        SimContract {
            messages_out: names![Shared],
            ..SimContract::EMPTY
        }
    }
    fn install(ctx: &mut SimInstaller) {
        ctx.emit_message::<Shared>();
        ctx.add_system(|| {});
    }
}

#[test]
#[should_panic(expected = "single-producer-messages")]
fn two_producers_same_message_panic() {
    let _ = Harness::new().with_sim::<A>().with_sim::<B>().build();
}
